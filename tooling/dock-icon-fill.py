#!/usr/bin/env python3
"""Require Dock icon artwork to cover the canvas.

macOS clips an app icon to a squircle. Pixels on that mask, including its
rim, have to be the bronze plate. A pale or transparent margin stays
visible as padding around a smaller rounded rect.

The cream mark is three dots in a column. A cream frame around an inset
plate is much wider than one dot. The 32px representation is the smallest
span that still counts.
"""

from __future__ import annotations

import struct
import sys
import zlib
from pathlib import Path

PNG_MAGIC = b"\x89PNG\r\n\x1a\n"
# Superellipse n=5 approximates the Dock mask. The check samples that rim;
# it does not draw a second shape into the artwork.
SQUIRCLE_POWER = 5.0
DOT_MIN_WIDTH = 0.18
DOT_MAX_WIDTH = 0.45
DOT_MIN_HEIGHT = 0.74
ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ICONS = (
    ROOT / "apps/desktop/src-tauri/icons/icon.icns",
    ROOT / "apps/desktop/src-tauri/icons/icon.png",
    ROOT / "apps/desktop/src-tauri/icons/128x128.png",
    ROOT / "apps/desktop/src-tauri/icons/128x128@2x.png",
)


def icns_pngs(data: bytes) -> list[bytes]:
    if len(data) < 8 or data[:4] != b"icns":
        raise ValueError("not an icns")
    images: list[bytes] = []
    offset = 8
    while offset + 8 <= len(data):
        length = int.from_bytes(data[offset + 4 : offset + 8], "big")
        if length < 8 or offset + length > len(data):
            break
        payload = data[offset + 8 : offset + length]
        if payload.startswith(PNG_MAGIC):
            images.append(payload)
        offset += length
    return images


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


def is_margin(pixel: tuple[int, int, int, int]) -> bool:
    _red, green, blue, alpha = pixel
    if alpha != 255:
        return True
    # Cream plate around the mark, near (245, 232, 216).
    return green >= 210 and blue >= 175 and _red >= 220


def is_dot(pixel: tuple[int, int, int, int]) -> bool:
    red, green, blue, alpha = pixel
    return alpha > 200 and red >= 230 and green >= 210 and blue >= 185


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


def bbox(width: int, height: int, rows: list[bytearray], predicate) -> tuple[int, int]:
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
        return 0, 0
    return max_x - min_x + 1, max_y - min_y + 1


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
    opaque_w, opaque_h = bbox(
        width, height, rows, lambda pixel: pixel[3] > 200
    )
    mark_w, mark_h = bbox(width, height, rows, is_dot)
    mark_wf = mark_w / width
    mark_hf = mark_h / height
    print(
        f"{label} {width}x{height} "
        f"opaque {opaque_w}x{opaque_h} ({opaque_w / width:.2%} x {opaque_h / height:.2%}) "
        f"dots {mark_w}x{mark_h} ({mark_wf:.2%} x {mark_hf:.2%})",
        flush=True,
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
        images = icns_pngs(data)
        if not images:
            return [f"{path} contains no png"]
        if not any(decode_png(image)[0] == 1024 for image in images):
            return [f"{path} has no 1024px image"]
        errors: list[str] = []
        for image in images:
            errors.extend(check_image(str(path), image))
        return errors
    return check_image(str(path), data)


def main(argv: list[str]) -> int:
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
