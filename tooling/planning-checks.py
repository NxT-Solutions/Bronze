#!/usr/bin/env python3
"""Planning-pack parity checks. Exit 0 only when every gate passes."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"

PRD_IDS = [
    *[f"G-{i:02d}" for i in range(1, 8)],
    *[f"CAP-{i:03d}" for i in range(1, 11)],
    *[f"QUE-{i:03d}" for i in range(1, 9)],
    *[f"WIN-{i:03d}" for i in range(1, 6)],
    *[f"DAT-{i:03d}" for i in range(1, 5)],
    *[f"SEC-{i:03d}" for i in range(1, 7)],
    *[f"A11Y-{i:03d}" for i in range(1, 7)],
    *[f"I18N-{i:03d}" for i in range(1, 5)],
    *[f"SET-{i:03d}" for i in range(1, 5)],
    "SUP-001",
    "SUP-002",
]

ENUMS = {
    "item_lifecycle": [
        "queued",
        "copied",
        "active",
        "done",
        "skipped",
        "trashed",
    ],
    "permission": [
        "unknown",
        "not_requested",
        "denied",
        "granted_unverified",
        "healthy",
        "degraded",
        "unavailable",
        "requires_relaunch",
    ],
    "post_copy": ["unchanged", "copied", "active", "done"],
    "advance": ["keep", "nextQueued"],
    "backup": ["daily", "weekly"],
    "capture_terminal": ["saved", "rejected", "failed", "cancelled"],
}

LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
ID_ROW_RE = re.compile(r"^\| (G-\d{2}|CAP-\d{3}|QUE-\d{3}|WIN-\d{3}|DAT-\d{3}|SEC-\d{3}|A11Y-\d{3}|I18N-\d{3}|SET-\d{3}|SUP-\d{3}) \|", re.M)


def fail(errors: list[str]) -> int:
    for item in errors:
        print(f"FAIL: {item}")
    print(f"{len(errors)} failure(s)")
    return 1


def check_requirement_parity() -> list[str]:
    errors: list[str] = []
    prd = (DOCS / "03-prd.md").read_text()
    backlog = (DOCS / "15-backlog-traceability.md").read_text()
    recon = (DOCS / "21-preimplementation-reconciliation.md").read_text()
    for req_id in PRD_IDS:
        if req_id not in prd:
            errors.append(f"PRD missing {req_id}")
        if req_id not in backlog:
            errors.append(f"traceability missing {req_id}")
        if req_id not in recon and req_id.startswith("G-"):
            continue
    backlog_ids = set(ID_ROW_RE.findall(backlog))
    row_ids = [req_id for req_id in PRD_IDS if not req_id.startswith("G-")]
    missing_rows = [req_id for req_id in row_ids if req_id not in backlog_ids]
    extra_rows = sorted(backlog_ids - set(PRD_IDS))
    for req_id in missing_rows:
        errors.append(f"traceability table missing row {req_id}")
    for req_id in extra_rows:
        errors.append(f"traceability table has unknown ID {req_id}")
    return errors


def check_enums() -> list[str]:
    errors: list[str] = []
    texts = {
        "04": (DOCS / "04-functional-spec.md").read_text(),
        "06": (DOCS / "06-system-architecture.md").read_text(),
        "07": (DOCS / "07-macos-capture-reliability.md").read_text(),
        "08": (DOCS / "08-data-model-and-portability.md").read_text(),
        "12": (DOCS / "12-settings-and-shortcuts.md").read_text(),
        "21": (DOCS / "21-preimplementation-reconciliation.md").read_text(),
    }
    for name in ENUMS["permission"]:
        if name not in texts["07"] or name not in texts["12"]:
            errors.append(f"permission enum {name} missing from 07 or 12")
    for name in ENUMS["item_lifecycle"]:
        if name not in texts["06"] or name not in texts["08"]:
            errors.append(f"lifecycle {name} missing from 06 or 08")
    for name in ENUMS["post_copy"]:
        if name not in texts["08"] or name not in texts["12"]:
            errors.append(f"post_copy {name} missing from 08 or 12")
    for name in ENUMS["advance"]:
        if name not in texts["08"] or name not in texts["12"]:
            errors.append(f"advance {name} missing from 08 or 12")
    for name in ENUMS["backup"]:
        if name not in texts["12"]:
            errors.append(f"backup schedule {name} missing from 12")
    if '"off"' in texts["12"] and 'backupSchedule: "daily" | "weekly"' not in texts["12"]:
        errors.append("backupSchedule lost daily|weekly contract")
    if "manual-only" in texts["12"] and "cannot be replaced by manual-only" not in texts["12"]:
        errors.append("manual-only backup prohibition missing")
    for name in ENUMS["capture_terminal"]:
        if name not in texts["07"]:
            errors.append(f"capture terminal {name} missing from 07")
    if "ShortcutActionId" not in texts["12"] or "capture.selection" not in texts["12"]:
        errors.append("shortcut action registry missing from settings schema")
    if "diagnostic_events" not in texts["04"]:
        errors.append("functional spec diagnostics not aligned to diagnostic_events")
    if "closed enum" not in texts["07"]:
        errors.append("permission closed-enum wording missing from 07")
    return errors


def resolve_link(source: Path, target: str) -> Path | None:
    href = target.split("#", 1)[0].split(" ", 1)[0].strip()
    if not href or href.startswith(("http://", "https://", "mailto:", "tel:")):
        return None
    if href.startswith("#"):
        return source
    return (source.parent / href).resolve()


def check_title_adr() -> list[str]:
    errors: list[str] = []
    adrs = (DOCS / "18-adrs.md").read_text()
    threat = (DOCS / "09-security-privacy-threat-model.md").read_text()
    start = adrs.find("## ADR-019:")
    if start < 0:
        return ["ADR-019 section missing"]
    nxt = adrs.find("\n## ", start + 1)
    section = adrs[start : nxt if nxt > 0 else None]
    if "Status: Proposed" not in section:
        errors.append("ADR-019 must stay Proposed")
    if "Status: Accepted" in section:
        errors.append("ADR-019 must not be Accepted")
    for needle in (
        "SmolLM2-135M-Instruct",
        "SmolLM2-360M-Instruct",
        "Qwen2.5-0.5B-Instruct",
        "llama-cpp-2",
        "compact_title",
        "Q4_K_M",
        "general.titleModel",
        "title-engine-status",
        "title_engine_status",
        "extractive",
        "Title:",
        "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d",
        "2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2",
        "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db",
    ):
        if needle not in section:
            errors.append(f"ADR-019 missing {needle}")
    if "Contents/Resources/models" not in section:
        errors.append("ADR-019 missing packaged models path")
    for adr_id, needles in (
        (
            "ADR-020",
            ("Status: Accepted", "titleCustomId", "GGUF", "Application Support"),
        ),
        (
            "ADR-021",
            ("Status: Accepted", "127.0.0.1", "/api/chat", "/api/pull"),
        ),
        (
            "ADR-022",
            ("Status: Accepted", "Keychain", "hosted-openai", "payload class"),
        ),
        (
            "ADR-023",
            (
                "Status: Proposed",
                "releases/latest",
                "brew upgrade --cask bronze",
                "No Sparkle",
                "user-initiated",
            ),
        ),
    ):
        start_n = adrs.find(f"## {adr_id}:")
        if start_n < 0:
            errors.append(f"{adr_id} section missing")
            continue
        nxt_n = adrs.find("\n## ", start_n + 1)
        body = adrs[start_n : nxt_n if nxt_n > 0 else None]
        for needle in needles:
            if needle not in body:
                errors.append(f"{adr_id} missing {needle}")
        if adr_id in ("ADR-020", "ADR-021", "ADR-022") and "| " + adr_id + " |" in adrs:
            row = next(
                (line for line in adrs.splitlines() if line.startswith(f"| {adr_id} |")),
                "",
            )
            if "Accepted" not in row:
                errors.append(f"{adr_id} index row must stay Accepted")
        if adr_id == "ADR-023":
            if re.search(r"^Status: Accepted", body, re.M):
                errors.append("ADR-023 must stay Proposed")
            row = next(
                (line for line in adrs.splitlines() if line.startswith("| ADR-023 |")),
                "",
            )
            if "Proposed" not in row:
                errors.append("ADR-023 index row must stay Proposed")
    t10 = next((line for line in threat.splitlines() if line.startswith("| T-10 ")), "")
    if not t10:
        errors.append("T-10 row missing")
    else:
        if "hosted AI" not in t10:
            errors.append("T-10 must still forbid hosted AI")
        if "Private Cloud Compute" not in t10:
            errors.append("T-10 must still forbid Private Cloud Compute")
        if "hash-pinned" not in t10 and "SHA-256-pinned" not in t10:
            errors.append("T-10 missing hash-pinned carve-out")
        if "allow-list" not in t10:
            errors.append("T-10 missing GGUF allow-list")
        if "SmolLM2-135M-Instruct-Q4_K_M.gguf" not in t10:
            errors.append("T-10 missing 135M filename")
        if "2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d" not in t10:
            errors.append("T-10 missing 135M hash")
        if "SmolLM2-360M-Instruct-Q4_K_M.gguf" not in t10:
            errors.append("T-10 missing 360M filename")
        if "2fa3f013dcdd7b99f9b237717fa0b12d75bbb89984cc1274be1471a465bac9c2" not in t10:
            errors.append("T-10 missing 360M hash")
        if "qwen2.5-0.5b-instruct-q4_k_m.gguf" not in t10:
            errors.append("T-10 missing Qwen filename")
        if "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db" not in t10:
            errors.append("T-10 missing Qwen hash")
        if "no bundled GGUF" in t10:
            errors.append("T-10 still bans bundled GGUF without carve-out")
    return errors


def check_oss_release() -> list[str]:
    errors: list[str] = []
    license_text = (ROOT / "LICENSE").read_text() if (ROOT / "LICENSE").is_file() else ""
    if not license_text:
        errors.append("LICENSE missing")
    else:
        if "MIT License" not in license_text:
            errors.append("LICENSE must be MIT")
        if "Noah Gillard" not in license_text:
            errors.append("LICENSE missing copyright holder")
        if "2026" not in license_text:
            errors.append("LICENSE missing copyright year")

    changelog = (ROOT / "CHANGELOG.md").read_text() if (ROOT / "CHANGELOG.md").is_file() else ""
    if not changelog:
        errors.append("CHANGELOG.md missing")
    elif "## Unreleased" not in changelog:
        errors.append("CHANGELOG.md missing Unreleased")

    for name in ("CONTRIBUTING.md", "SECURITY.md"):
        if not (ROOT / name).is_file():
            errors.append(f"{name} missing")

    contributing = (ROOT / "CONTRIBUTING.md").read_text() if (ROOT / "CONTRIBUTING.md").is_file() else ""
    if contributing and "NxT-Solutions/homebrew-nxt-solutions-packages" not in contributing:
        errors.append("CONTRIBUTING.md must name the org Homebrew tap")
    if contributing and "NoahNxT/homebrew-nxt-solutions-packages" in contributing:
        errors.append("CONTRIBUTING.md must not name the personal Homebrew tap")

    security = (ROOT / "SECURITY.md").read_text() if (ROOT / "SECURITY.md").is_file() else ""
    if security and "NxT-Solutions/Bronze/security/advisories" not in security:
        errors.append("SECURITY.md missing GitHub Advisories URL")

    owners = ROOT / ".github" / "CODEOWNERS"
    if not owners.is_file():
        errors.append(".github/CODEOWNERS missing")
    elif "@NoahNxT" not in owners.read_text():
        errors.append(".github/CODEOWNERS must name @NoahNxT")

    release = ROOT / ".github" / "workflows" / "release.yml"
    release_text = release.read_text() if release.is_file() else ""
    if not release_text:
        errors.append(".github/workflows/release.yml missing")
    else:
        for needle in (
            "workflow_dispatch",
            "pkgbuild",
            "app.bronze.desktop",
            "bronze-macos-arm64.pkg",
            "bronze-macos-x86_64.pkg",
            "aarch64-apple-darwin",
            "x86_64-apple-darwin",
            "macos-15",
            "macos-15-intel",
            "This package is not a notarization claim.",
        ):
            if needle not in release_text:
                errors.append(f"release.yml missing {needle}")
        if "git push origin main" in release_text or "git push origin HEAD" in release_text:
            errors.append("release.yml must not push main")
        if "files: dist/**/*" in release_text:
            errors.append(
                "release.yml must not upload dist/**/* "
                "(duplicate NOTARIZATION.txt basenames 404 on asset delete)"
            )
        if "files: staged/*" not in release_text:
            errors.append("release.yml must upload unique staged/* release files")

    brew = ROOT / ".github" / "workflows" / "publish-homebrew.yml"
    brew_text = brew.read_text() if brew.is_file() else ""
    if not brew_text:
        errors.append(".github/workflows/publish-homebrew.yml missing")
    else:
        if "NxT-Solutions/homebrew-nxt-solutions-packages" not in brew_text:
            errors.append(
                "publish-homebrew.yml must clone NxT-Solutions/homebrew-nxt-solutions-packages"
            )
        if "NoahNxT/homebrew-nxt-solutions-packages" in brew_text:
            errors.append("publish-homebrew.yml must not clone the personal tap")
        if "Casks/bronze.rb" not in brew_text:
            errors.append("publish-homebrew.yml must write Casks/bronze.rb")
        if 'cask "bronze"' not in brew_text:
            errors.append("publish-homebrew.yml must emit a Homebrew cask")
        if "class Bronze < Formula" in brew_text:
            errors.append("publish-homebrew.yml must not write Formula/bronze.rb")
        if "brew upgrade --cask bronze" not in brew_text and "pkgutil" not in brew_text:
            errors.append("publish-homebrew.yml missing cask uninstall identity")
        if "on_arm" not in brew_text:
            errors.append("publish-homebrew.yml must emit on_arm")
        if "on_intel" not in brew_text:
            errors.append("publish-homebrew.yml must emit on_intel")
        if "bronze-macos-arm64.pkg" not in brew_text:
            errors.append("publish-homebrew.yml must download bronze-macos-arm64.pkg")
        if "bronze-macos-x86_64.pkg" not in brew_text:
            errors.append("publish-homebrew.yml must download bronze-macos-x86_64.pkg")
        if re.search(r"if:.*secrets\.", brew_text):
            errors.append("publish-homebrew.yml must not use secrets in if:")

    adrs_text = (DOCS / "18-adrs.md").read_text() if (DOCS / "18-adrs.md").is_file() else ""
    start_002 = adrs_text.find("## ADR-002:")
    if start_002 < 0:
        errors.append("ADR-002 section missing")
    else:
        nxt_002 = adrs_text.find("\n## ", start_002 + 1)
        body_002 = adrs_text[start_002 : nxt_002 if nxt_002 > 0 else None]
        if "Status: Accepted" not in body_002:
            errors.append("ADR-002 must be Accepted")
        if "operator explicitly asked" not in body_002:
            errors.append("ADR-002 must record that the operator asked for Intel")
        if "bronze-macos-arm64.pkg" not in body_002 or "bronze-macos-x86_64.pkg" not in body_002:
            errors.append("ADR-002 must name both split packages")
        row_002 = next(
            (line for line in adrs_text.splitlines() if line.startswith("| ADR-002 |")),
            "",
        )
        if "Accepted" not in row_002:
            errors.append("ADR-002 index row must be Accepted")

    readme = (ROOT / "README.md").read_text() if (ROOT / "README.md").is_file() else ""
    if "brew tap NxT-Solutions/nxt-solutions-packages" not in readme:
        errors.append("README must tap NxT-Solutions/nxt-solutions-packages")
    if "brew tap NoahNxT/nxt-solutions-packages" in readme:
        errors.append("README must not tap NoahNxT/nxt-solutions-packages")
    for image in (
        "docs/images/bronze-hero.png",
        "docs/images/bronze-queue.png",
        "docs/images/bronze-settings.png",
        "docs/images/bronze-mark.png",
    ):
        if image not in readme:
            errors.append(f"README missing {image}")
        if not (ROOT / image).is_file():
            errors.append(f"{image} missing")

    ci = ROOT / ".github" / "workflows" / "ci.yml"
    ci_text = ci.read_text() if ci.is_file() else ""
    if not ci_text:
        errors.append(".github/workflows/ci.yml missing")
    else:
        for needle in (
            "pull_request",
            "contents: read",
            "pnpm exec biome check .",
            "tooling/planning-checks.py",
            "--exclude bronze-desktop",
        ):
            if needle not in ci_text:
                errors.append(f"ci.yml missing {needle}")
        if "notarize" in ci_text.lower() and "does not notarize" not in ci_text.lower():
            errors.append("ci.yml must not run notarization")

    dependabot = ROOT / ".github" / "dependabot.yml"
    if not dependabot.is_file():
        errors.append(".github/dependabot.yml missing")
    else:
        dep = dependabot.read_text()
        for eco in ("github-actions", "npm", "cargo"):
            if eco not in dep:
                errors.append(f"dependabot.yml missing {eco}")
    return errors


def check_local_links() -> list[str]:
    errors: list[str] = []
    roots = [
        ROOT / "README.md",
        ROOT / "AGENTS.md",
        *sorted((ROOT / "docs").glob("*.md")),
        *sorted((ROOT / "research").glob("*.md")),
        ROOT / "assets" / "README.md",
    ]
    for path in roots:
        text = path.read_text()
        for match in LINK_RE.finditer(text):
            dest = resolve_link(path, match.group(2))
            if dest is None:
                continue
            if not dest.exists():
                errors.append(f"{path.relative_to(ROOT)} -> {match.group(2)}")
    return errors


def main() -> int:
    errors: list[str] = []
    errors.extend(check_requirement_parity())
    errors.extend(check_enums())
    errors.extend(check_local_links())
    errors.extend(check_title_adr())
    errors.extend(check_oss_release())
    if errors:
        return fail(errors)
    print("PASS requirement-parity")
    print("PASS enum/schema")
    print("PASS local-links")
    print("PASS traceability")
    print("PASS title-adr")
    print("PASS oss-release")
    print(f"checked {len(PRD_IDS)} requirement IDs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
