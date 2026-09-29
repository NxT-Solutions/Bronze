#!/usr/bin/env python3
"""Require Dock icon artwork to be a flat opaque plate.

macOS clips the Dock tile to a squircle. Get Info's preview is a rounded
well of the raw asset. A second rounded rect — even an opaque one — reads
as a grey box. Every pixel, including the corners macOS later clips, has
to be the bronze plate or a cream dot. No alpha, no cream frame.

The cream mark is three dots in a column. A cream frame around an inset
plate is much wider than one dot. The 16px face is the smallest span
that still counts: Notification Center asks icon services for 16px and
32px, and those faces must be PNG. Packed RGB slots (il32 / is32) are a
different image and read as a cream well around the mark.

32x32.png is the menu-bar template and is not in this check.
"""

from __future__ import annotations

import math
import shutil
import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

PNG_MAGIC = b"\x89PNG\r\n\x1a\n"
SQUIRCLE_POWER = 5.0
DOT_MIN_WIDTH = 0.18
DOT_MAX_WIDTH = 0.45
DOT_MIN_HEIGHT = 0.74
PLATE_DEV = 8
RESIZE_DEV = 18
BLEND_PAD_FRAC = 0.02
MASTER_SIZE = 1024
MASTER_FILL = (196, 143, 98, 255)
MASTER_DOT = (245, 232, 216, 255)
MASTER_RADIUS = 124.0
MASTER_CENTERS = ((511.5, 223.0), (511.5, 511.5), (511.5, 802.5))
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ICONS = (
    ROOT / "apps/desktop/src-tauri/icons/icon.icns",
    ROOT / "apps/desktop/src-tauri/icons/icon.png",
    ROOT / "apps/desktop/src-tauri/icons/128x128.png",
    ROOT / "apps/desktop/src-tauri/icons/128x128@2x.png",
)
NOTICE_PNG_SIZES = (16, 32)
NOTICE_ICONSET = ("icon_16x16@2x.png", "icon_32x32@2x.png")
LEGACY_ICNS = (b"il32", b"is32", b"l8mk", b"s8mk", b"ic04", b"ic05")


def icns_entries(data: bytes) -> list[tuple[bytes, bytes]]:
    if len(data) < 8 or data[:4] != b"icns":
        raise ValueError("not an icns")
    entries: list[tuple[bytes, bytes]] = []
    offset = 8
    while offset + 8 <= len(data):
        kind = data[offset : offset + 4]
        length = int.from_bytes(data[offset + 4 : offset + 8], "big")
        if length < 8 or offset + length > len(data):
            break
        entries.append((kind, data[offset + 8 : offset + length]))
        offset += length
    return entries


def icns_pngs(data: bytes) -> list[bytes]:
    return [payload for _kind, payload in icns_entries(data) if payload.startswith(PNG_MAGIC)]


def write_icns(path: Path, entries: list[tuple[bytes, bytes]]) -> None:
    parts = bytearray()
    for kind, payload in entries:
        if len(kind) != 4:
            raise ValueError(f"icns type {kind!r}")
        length = 8 + len(payload)
        parts.extend(kind)
        parts.extend(length.to_bytes(4, "big"))
        parts.extend(payload)
    path.write_bytes(b"icns" + (8 + len(parts)).to_bytes(4, "big") + parts)


def rebuild_notice_faces(path: Path) -> None:
    """Keep large PNG faces. Replace the 16/32 banner faces with the master.

    Packed RGB/ARGB slots are a different image. Notification Center prefers
    those slots for the 16px and 32px faces, so they have to go.
    """
    kept: list[tuple[bytes, bytes]] = []
    for kind, payload in icns_entries(path.read_bytes()):
        if kind in LEGACY_ICNS:
            continue
        if payload.startswith(PNG_MAGIC):
            width = int.from_bytes(payload[16:20], "big")
            if width <= 32:
                continue
        kept.append((kind, payload))
    kept.append((b"icp4", encode_png(16, 16, paint_master(16))))
    kept.append((b"icp5", encode_png(32, 32, paint_master(32))))
    kept.append((b"ic11", encode_png(32, 32, paint_master(32))))
    write_icns(path, kept)


def iconutil_pngs(path: Path) -> list[tuple[str, bytes]]:
    iconutil = shutil.which("iconutil")
    if iconutil is None:
        return []
    work = Path(tempfile.mkdtemp(prefix="bronze-iconset-"))
    iconset = work / "App.iconset"
    try:
        built = subprocess.run(
            [iconutil, "-c", "iconset", str(path), "-o", str(iconset)],
            check=False,
            capture_output=True,
            text=True,
        )
        if built.returncode != 0 or not iconset.is_dir():
            raise ValueError(
                f"iconutil could not extract {path}: {built.stderr.strip() or built.stdout.strip()}"
            )
        images: list[tuple[str, bytes]] = []
        for name in NOTICE_ICONSET:
            image = iconset / name
            if image.is_file():
                images.append((name, image.read_bytes()))
        return images
    finally:
        shutil.rmtree(work, ignore_errors=True)


def decode_png(data: bytes) -> tuple[int, int, list[bytearray]]:
    if not data.startswith(PNG_MAGIC):
        raise ValueError("not a png")
    pos = 8
    width = height = None
    bit_depth = color_type = None
    idat = b""
    while pos < len(data):
        length = struct.unpack(">I", data[pos : pos + 4])[0]
        kind = data[pos + 4 : pos + 8]
        chunk = data[pos + 8 : pos + 8 + length]
        pos += 12 + length
        if kind == b"IHDR":
            width, height, bit_depth, color_type = struct.unpack(">IIBB", chunk[:10])
        elif kind == b"IDAT":
            idat += chunk
        elif kind == b"IEND":
            break
    if width is None or height is None or bit_depth != 8 or color_type != 6:
        raise ValueError(f"unsupported png {width}x{height} depth={bit_depth} color={color_type}")
    raw = zlib.decompress(idat)
    stride = width * 4
    rows: list[bytearray] = []
    index = 0
    prev = bytearray(stride)

    def paeth(left: int, up: int, up_left: int) -> int:
        estimate = left + up - up_left
        pa, pb, pc = abs(estimate - left), abs(estimate - up), abs(estimate - up_left)
        if pa <= pb and pa <= pc:
            return left
        if pb <= pc:
            return up
        return up_left

    for _y in range(height):
        filt = raw[index]
        index += 1
        row = bytearray(raw[index : index + stride])
        index += stride
        if filt == 1:
            for x in range(stride):
                left = row[x - 4] if x >= 4 else 0
                row[x] = (row[x] + left) & 255
        elif filt == 2:
            for x in range(stride):
                row[x] = (row[x] + prev[x]) & 255
        elif filt == 3:
            for x in range(stride):
                left = row[x - 4] if x >= 4 else 0
                row[x] = (row[x] + ((left + prev[x]) // 2)) & 255
        elif filt == 4:
            for x in range(stride):
                left = row[x - 4] if x >= 4 else 0
                up_left = prev[x - 4] if x >= 4 else 0
                row[x] = (row[x] + paeth(left, prev[x], up_left)) & 255
        elif filt != 0:
            raise ValueError(f"png filter {filt}")
        rows.append(row)
        prev = row
    return width, height, rows


def encode_png(width: int, height: int, rows: list[bytearray]) -> bytes:
    def chunk(tag: bytes, payload: bytes) -> bytes:
        crc = zlib.crc32(tag + payload) & 0xFFFFFFFF
        return struct.pack(">I", len(payload)) + tag + payload + struct.pack(">I", crc)

    raw = b"".join(b"\x00" + bytes(row) for row in rows)
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (
        PNG_MAGIC
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def is_margin(pixel: tuple[int, int, int, int]) -> bool:
    _red, green, blue, alpha = pixel
    if alpha != 255:
        return True
    return green >= 210 and blue >= 175 and _red >= 220


def is_dot(pixel: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = pixel
    return alpha == 255 and red >= 230 and green >= 210 and blue >= 185


def near_plate(
    pixel: tuple[int, int, int, int], plate: tuple[int, int, int, int], dev: int = PLATE_DEV
) -> bool:
    if pixel[3] != 255 or plate[3] != 255:
        return False
    return all(abs(pixel[i] - plate[i]) <= dev for i in range(3))


def is_blend(
    pixel: tuple[int, int, int, int],
    plate: tuple[int, int, int, int],
    cream: tuple[int, int, int, int],
) -> bool:
    if pixel[3] != 255:
        return False
    for i in range(3):
        lo = min(plate[i], cream[i])
        hi = max(plate[i], cream[i])
        if not (lo <= pixel[i] <= hi):
            return False
    return True


def in_squircle(x: int, y: int, width: int, height: int) -> bool:
    nx = 2 * ((x + 0.5) / width) - 1
    ny = 2 * ((y + 0.5) / height) - 1
    return abs(nx) ** SQUIRCLE_POWER + abs(ny) ** SQUIRCLE_POWER <= 1.0


def on_squircle_rim(x: int, y: int, width: int, height: int) -> bool:
    if not in_squircle(x, y, width, height):
        return False
    for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
        nx, ny = x + dx, y + dy
        if nx < 0 or ny < 0 or nx >= width or ny >= height:
            return True
        if not in_squircle(nx, ny, width, height):
            return True
    return False


def bbox_rect(
    width: int, height: int, rows: list[bytearray], predicate
) -> tuple[int, int, int, int] | None:
    min_x, min_y, max_x, max_y = width, height, -1, -1
    for y in range(height):
        row = rows[y]
        for x in range(width):
            pixel = tuple(row[x * 4 : x * 4 + 4])
            if predicate(pixel):
                min_x = min(min_x, x)
                min_y = min(min_y, y)
                max_x = max(max_x, x)
                max_y = max(max_y, y)
    if max_x < 0:
        return None
    return min_x, min_y, max_x, max_y


def bbox(width: int, height: int, rows: list[bytearray], predicate) -> tuple[int, int]:
    rect = bbox_rect(width, height, rows, predicate)
    if rect is None:
        return 0, 0
    min_x, min_y, max_x, max_y = rect
    return max_x - min_x + 1, max_y - min_y + 1


def lerp_channel(a: int, b: int, t: float) -> int:
    return int(round(a + (b - a) * t))


def paint_master(size: int = MASTER_SIZE) -> list[bytearray]:
    scale = size / MASTER_SIZE
    radius = MASTER_RADIUS * scale
    centers = tuple((cx * scale, cy * scale) for cx, cy in MASTER_CENTERS)
    rows: list[bytearray] = []
    for y in range(size):
        row = bytearray(size * 4)
        for x in range(size):
            cover = 0.0
            for cx, cy in centers:
                dist = math.hypot(x + 0.5 - cx, y + 0.5 - cy)
                cover = max(cover, min(1.0, max(0.0, radius + 0.65 - dist)))
            if cover <= 0:
                pixel = MASTER_FILL
            elif cover >= 1:
                pixel = MASTER_DOT
            else:
                pixel = (
                    lerp_channel(MASTER_FILL[0], MASTER_DOT[0], cover),
                    lerp_channel(MASTER_FILL[1], MASTER_DOT[1], cover),
                    lerp_channel(MASTER_FILL[2], MASTER_DOT[2], cover),
                    255,
                )
            row[x * 4 : x * 4 + 4] = pixel
        rows.append(row)
    return rows


def write_master(path: Path) -> None:
    rows = paint_master(MASTER_SIZE)
    path.write_bytes(encode_png(MASTER_SIZE, MASTER_SIZE, rows))


def check_image(label: str, data: bytes) -> list[str]:
    width, height, rows = decode_png(data)
    errors: list[str] = []
    points = (
        (0, 0),
        (width - 1, 0),
        (0, height - 1),
        (width - 1, height - 1),
        (0, height // 2),
        (width - 1, height // 2),
        (width // 2, 0),
        (width // 2, height - 1),
    )
    alpha_bad = 0
    for y in range(height):
        row = rows[y]
        for x in range(width):
            if row[x * 4 + 3] != 255:
                alpha_bad += 1
                if alpha_bad <= 4:
                    pixel = tuple(row[x * 4 : x * 4 + 4])
                    errors.append(f"{label} {width}x{height} ({x},{y}) is not opaque {pixel}")
    if alpha_bad > 4:
        errors.append(f"{label} {width}x{height} has {alpha_bad} pixels with alpha")
    for x, y in points:
        pixel = tuple(rows[y][x * 4 : x * 4 + 4])
        if is_margin(pixel):
            errors.append(f"{label} {width}x{height} ({x},{y}) is margin {pixel}")
    rim_bad = 0
    band = max(1, int(min(width, height) * 0.2))
    for y in range(height):
        row = rows[y]
        for x in range(width):
            if band <= x < width - band and band <= y < height - band:
                continue
            on_edge = x in (0, width - 1) or y in (0, height - 1)
            if not on_edge and not on_squircle_rim(x, y, width, height):
                continue
            pixel = tuple(row[x * 4 : x * 4 + 4])
            if not is_margin(pixel):
                continue
            rim_bad += 1
            if rim_bad <= 4 and (x, y) not in points:
                errors.append(f"{label} {width}x{height} ({x},{y}) is margin {pixel}")
    if rim_bad > 4:
        errors.append(
            f"{label} {width}x{height} has {rim_bad} margin pixels on the squircle rim"
        )
    plate = tuple(rows[0][0:4])
    if is_margin(plate):
        errors.append(f"{label} {width}x{height} corner plate is margin {plate}")
    cream = MASTER_DOT
    dot_rect = bbox_rect(width, height, rows, is_dot)
    pad = max(2, int(min(width, height) * BLEND_PAD_FRAC))
    if dot_rect is None:
        pad_rect = None
    else:
        min_x, min_y, max_x, max_y = dot_rect
        pad_rect = (
            max(0, min_x - pad),
            max(0, min_y - pad),
            min(width - 1, max_x + pad),
            min(height - 1, max_y + pad),
        )
    stray = 0
    for y in range(height):
        row = rows[y]
        for x in range(width):
            pixel = tuple(row[x * 4 : x * 4 + 4])
            if near_plate(pixel, plate) or is_dot(pixel):
                continue
            inside_pad = (
                pad_rect is not None
                and pad_rect[0] <= x <= pad_rect[2]
                and pad_rect[1] <= y <= pad_rect[3]
            )
            if inside_pad and (
                is_blend(pixel, plate, cream) or near_plate(pixel, plate, RESIZE_DEV)
            ):
                continue
            stray += 1
            if stray <= 4:
                errors.append(
                    f"{label} {width}x{height} ({x},{y}) is not a flat plate {pixel}"
                )
    if stray > 4:
        errors.append(
            f"{label} {width}x{height} has {stray} pixels off the flat plate"
        )
    opaque_w, opaque_h = bbox(width, height, rows, lambda pixel: pixel[3] == 255)
    mark_w, mark_h = bbox(width, height, rows, is_dot)
    mark_wf = mark_w / width
    mark_hf = mark_h / height
    print(
        f"{label} {width}x{height} "
        f"opaque {opaque_w}x{opaque_h} ({opaque_w / width:.2%} x {opaque_h / height:.2%}) "
        f"dots {mark_w}x{mark_h} ({mark_wf:.2%} x {mark_hf:.2%})",
        flush=True,
    )
    if opaque_w != width or opaque_h != height:
        errors.append(
            f"{label} {width}x{height} opaque bbox {opaque_w}x{opaque_h} is not the canvas"
        )
    if mark_hf < DOT_MIN_HEIGHT or not (DOT_MIN_WIDTH <= mark_wf <= DOT_MAX_WIDTH):
        errors.append(
            f"{label} {width}x{height} dot span {mark_wf:.2%} x {mark_hf:.2%} "
            f"is outside {DOT_MIN_WIDTH:.0%}–{DOT_MAX_WIDTH:.0%} by {DOT_MIN_HEIGHT:.0%}"
        )
    return errors


def check_path(path: Path) -> list[str]:
    data = path.read_bytes()
    if data[:4] == b"icns":
        entries = icns_entries(data)
        images = [payload for _kind, payload in entries if payload.startswith(PNG_MAGIC)]
        if not images:
            return [f"{path} contains no png"]
        sizes = [decode_png(image)[0] for image in images]
        errors: list[str] = []
        if 1024 not in sizes:
            errors.append(f"{path} has no 1024px image")
        for need in NOTICE_PNG_SIZES:
            if need not in sizes:
                errors.append(
                    f"{path} has no {need}px png; Notification Center uses that face"
                )
        for kind, _payload in entries:
            if kind in LEGACY_ICNS:
                errors.append(
                    f"{path} still has {kind.decode()}; "
                    "Notification Center prefers that slot over the PNG face"
                )
        for image in images:
            errors.extend(check_image(str(path), image))
        try:
            extracted = iconutil_pngs(path)
        except ValueError as exc:
            errors.append(str(exc))
            extracted = []
        if extracted:
            found = {name for name, _image in extracted}
            for name in NOTICE_ICONSET:
                if name not in found:
                    errors.append(f"{path} iconutil extract is missing {name}")
            for name, image in extracted:
                errors.extend(check_image(f"{path} {name}", image))
        return errors
    return check_image(str(path), data)


def _fill_circle(
    rows: list[bytearray],
    cx: float,
    cy: float,
    radius: float,
    color: tuple[int, int, int, int],
) -> None:
    height = len(rows)
    width = len(rows[0]) // 4
    y0 = max(0, int(cy - radius - 1))
    y1 = min(height - 1, int(cy + radius + 1))
    x0 = max(0, int(cx - radius - 1))
    x1 = min(width - 1, int(cx + radius + 1))
    pixel = bytes(color)
    r2 = radius * radius
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if (x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2 <= r2:
                rows[y][x * 4 : x * 4 + 4] = pixel


def self_test() -> list[str]:
    errors: list[str] = []
    size = 64
    flat = paint_master(size)
    if check_image("self-test flat", encode_png(size, size, flat)):
        errors.append("self-test: flat full-bleed master must pass")

    rounded = [bytearray(MASTER_FILL) * size for _ in range(size)]
    for y in range(size):
        for x in range(size):
            nx = 2 * ((x + 0.5) / size) - 1
            ny = 2 * ((y + 0.5) / size) - 1
            if abs(nx) ** 5 + abs(ny) ** 5 <= 0.72:
                rounded[y][x * 4 : x * 4 + 4] = bytes((173, 121, 78, 255))
    _fill_circle(rounded, 31.5, 16.0, 7.5, MASTER_DOT)
    _fill_circle(rounded, 31.5, 31.5, 7.5, MASTER_DOT)
    _fill_circle(rounded, 31.5, 47.0, 7.5, MASTER_DOT)
    if not check_image("self-test inset plate", encode_png(size, size, rounded)):
        errors.append("self-test: rounded inset plate must fail")

    punched = paint_master(size)
    clear = bytes((196, 143, 98, 0))
    punched[0][0:4] = clear
    punched[0][-4:] = clear
    punched[-1][0:4] = clear
    punched[-1][-4:] = clear
    if not check_image("self-test alpha corners", encode_png(size, size, punched)):
        errors.append("self-test: transparent corners must fail")

    notice16 = paint_master(16)
    if check_image("self-test notice 16", encode_png(16, 16, notice16)):
        errors.append("self-test: 16px full-bleed master must pass")
    notice32 = paint_master(32)
    if check_image("self-test notice 32", encode_png(32, 32, notice32)):
        errors.append("self-test: 32px full-bleed master must pass")
    return errors


def main(argv: list[str]) -> int:
    if len(argv) >= 3 and argv[1] == "--write-master":
        write_master(Path(argv[2]))
        return 0
    if len(argv) >= 2 and argv[1] == "--rebuild-icns":
        target = Path(argv[2]) if len(argv) >= 3 else DEFAULT_ICONS[0]
        rebuild_notice_faces(target)
        return 0
    self_errors = self_test()
    if self_errors:
        for error in self_errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1
    if len(argv) >= 2 and argv[1] == "--self-test":
        return 0
    paths = [Path(arg) for arg in argv[1:]] or list(DEFAULT_ICONS)
    errors: list[str] = []
    for path in paths:
        if not path.is_file():
            errors.append(f"missing {path}")
            continue
        errors.extend(check_path(path))
    if errors:
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
