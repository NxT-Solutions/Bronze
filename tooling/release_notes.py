#!/usr/bin/env python3
"""Plain-language What's new blocks for the changelog and GitHub release."""

from __future__ import annotations

import datetime
import os
import re
import subprocess
import sys
from pathlib import Path

MAX_NOTES = 5
NOTE_CHECK_FOR_UPDATES = "Check for updates is easier to see."
NOTE_DOCK_ICON = "The Dock icon fills its tile."
NOTE_ENGINE_SWITCH = (
    "Switching the on-this-Mac title engine finishes instead of staying on Loading."
)
NOTE_PERMISSIONS = "Bronze asks once for the macOS permissions it uses."
CONVENTIONAL_TYPES = {
    "feat",
    "fix",
    "chore",
    "docs",
    "style",
    "refactor",
    "perf",
    "test",
    "build",
    "ci",
    "revert",
}
COMMIT_VERBS = CONVENTIONAL_TYPES | {"add", "update", "set", "bump", "wip"}


def clean_note_line(raw: str) -> str:
    text = raw.strip()
    for marker in ("- ", "* ", "• "):
        if text.startswith(marker):
            text = text[len(marker) :].strip()
            break
    text = re.sub(r"\[([^\]]+)\]\([^)]*\)", r"\1", text)
    text = text.replace("**", "").replace("__", "").replace("`", "")
    words = []
    for word in text.split():
        lower = word.lower()
        if "https://" in lower or "http://" in lower:
            continue
        words.append(word)
    kept: list[str] = []
    index = 0
    while index < len(words):
        if (
            words[index].lower() == "by"
            and index + 1 < len(words)
            and words[index + 1].startswith("@")
        ):
            index += 2
            if index < len(words) and words[index].lower() == "in":
                index += 1
            continue
        kept.append(words[index])
        index += 1
    return " ".join(kept).strip()


def conventional_commit(line: str) -> tuple[str, str] | None:
    if ":" not in line:
        return None
    head, desc = line.split(":", 1)
    head = head.strip()
    desc = desc.strip()
    if not desc or " " in head:
        return None
    type_name = head.split("(", 1)[0].rstrip("!")
    if not type_name.isalpha() or type_name.lower() not in CONVENTIONAL_TYPES:
        return None
    if "(" in head:
        if ")" not in head:
            return None
        close = head.find(")")
        after = head[close + 1 :]
        if after not in ("", "!"):
            return None
    elif "!" in head and not head.endswith("!"):
        return None
    return type_name.lower(), desc


def map_user_change(kind: str, desc: str) -> str | None:
    if kind not in {"feat", "fix"}:
        return None
    description = desc.strip().rstrip(".").lower()
    if "set version" in description or "bump version" in description:
        return None
    if "check for updates" in description and "push button" in description:
        return NOTE_CHECK_FOR_UPDATES
    if "dock" in description and (
        "canvas" in description or "bleed" in description or "fills" in description
    ):
        return NOTE_DOCK_ICON
    if (
        ("on-this-mac" in description or "on this mac" in description)
        and "engine" in description
        and ("switch" in description or "settle" in description)
    ):
        return NOTE_ENGINE_SWITCH
    if "permission" in description and (
        "once" in description or "prompt" in description
    ):
        return NOTE_PERMISSIONS
    return None


def is_version_line(line: str) -> bool:
    lower = line.lower()
    return lower.startswith("version ") and "is available" in lower


def is_changelog_trailer(line: str) -> bool:
    normalized = line.replace("’", "'").lower()
    return (
        normalized.startswith("full changelog")
        or normalized.startswith("what's changed")
        or normalized == "changes"
    )


def looks_like_commit_subject(line: str) -> bool:
    if line.endswith("."):
        return False
    first = line.split(None, 1)[0].rstrip(":").lower() if line.split() else ""
    return first in COMMIT_VERBS


def summary_line(raw: str) -> str | None:
    cleaned = clean_note_line(raw)
    if not cleaned or is_version_line(cleaned) or is_changelog_trailer(cleaned):
        return None
    parsed = conventional_commit(cleaned)
    if parsed:
        return map_user_change(*parsed)
    if "@" in cleaned:
        return None
    if looks_like_commit_subject(cleaned) or " " not in cleaned:
        return None
    return cleaned


def plain_notes(lines: list[str]) -> list[str]:
    notes: list[str] = []
    for raw in lines:
        if len(notes) >= MAX_NOTES:
            break
        note = summary_line(raw)
        if note and note not in notes:
            notes.append(note)
    return notes


def _heading_title(line: str) -> str | None:
    stripped = line.strip()
    if not stripped.startswith("#"):
        return None
    title = stripped.lstrip("#").strip()
    return title or None


def release_notes_from_markdown(markdown: str) -> list[str]:
    lines = markdown.splitlines()
    start = None
    for index, line in enumerate(lines):
        title = _heading_title(line)
        if title and _is_whats_new(title):
            start = index + 1
            break
    if start is not None:
        section: list[str] = []
        for line in lines[start:]:
            if _heading_title(line):
                break
            section.append(line)
        return _take(section, from_whats_new=True)
    return _take(lines, from_whats_new=False)


def _is_whats_new(title: str) -> bool:
    normalized = title.strip().rstrip(":").replace("’", "'").lower()
    return normalized in {"what's new", "whats new"}


def _take(lines: list[str], from_whats_new: bool) -> list[str]:
    notes: list[str] = []
    for raw in lines:
        if len(notes) >= MAX_NOTES:
            break
        note = _note_from_line(raw, from_whats_new)
        if note and note not in notes:
            notes.append(note)
    return notes


def _note_from_line(raw: str, from_whats_new: bool) -> str | None:
    if not raw.strip() or _heading_title(raw):
        return None
    cleaned = clean_note_line(raw)
    if not cleaned or is_version_line(cleaned) or is_changelog_trailer(cleaned):
        return None
    parsed = conventional_commit(cleaned)
    if parsed:
        return map_user_change(*parsed)
    if "@" in cleaned:
        return None
    if not from_whats_new and (looks_like_commit_subject(cleaned) or " " not in cleaned):
        return None
    return cleaned


def human_unreleased_notes(lines: list[str]) -> list[str]:
    notes: list[str] = []
    in_section = False
    for line in lines:
        if line.strip() == "## Unreleased":
            in_section = True
            continue
        if in_section and line.startswith("## "):
            break
        if not in_section or not line.strip().startswith(("- ", "* ", "• ")):
            continue
        text = clean_note_line(line)
        if not text or text.lower() == "work in progress.":
            continue
        note = _note_from_line(text, from_whats_new=True)
        if note and note not in notes and len(notes) < MAX_NOTES:
            notes.append(note)
    return notes


def render_version_section(
    version: str, today: str, whats_new: list[str], subjects: list[str]
) -> list[str]:
    lines = [f"## v{version} - {today}", "", "### What's new", ""]
    lines.extend(f"- {note}" for note in whats_new)
    changes = subjects or ["Internal maintenance updates."]
    lines.extend(["", "### Changes", ""])
    lines.extend(f"- {subject}" for subject in changes)
    lines.append("")
    return lines


def _drop_version(lines: list[str], version: str) -> list[str]:
    header = re.compile(rf"^## v{re.escape(version)} - \d{{4}}-\d{{2}}-\d{{2}}$")
    cleaned: list[str] = []
    index = 0
    while index < len(lines):
        if header.match(lines[index].strip()):
            index += 1
            while index < len(lines) and not lines[index].startswith("## "):
                index += 1
            continue
        cleaned.append(lines[index])
        index += 1
    return cleaned


def _reset_unreleased(lines: list[str]) -> list[str]:
    out: list[str] = []
    index = 0
    replaced = False
    while index < len(lines):
        if lines[index].strip() == "## Unreleased" and not replaced:
            out.extend(["## Unreleased", "", "- Work in progress.", ""])
            index += 1
            while index < len(lines) and not lines[index].startswith("## "):
                index += 1
            replaced = True
            continue
        out.append(lines[index])
        index += 1
    if not replaced:
        if out and out[-1].strip():
            out.append("")
        out.extend(["## Unreleased", "", "- Work in progress.", ""])
    return out


def update_changelog_text(
    text: str, version: str, today: str, subjects: list[str]
) -> str:
    if re.search(
        rf"^## v{re.escape(version)} - \d{{4}}-\d{{2}}-\d{{2}}$",
        text,
        re.M,
    ):
        return text
    lines = text.splitlines()
    whats_new = human_unreleased_notes(lines) or plain_notes(subjects)
    lines = _reset_unreleased(lines)
    lines = _drop_version(lines, version)
    if not any(line.strip() == "## Unreleased" for line in lines):
        lines = _reset_unreleased(lines)
    insert_at = len(lines)
    for index, line in enumerate(lines):
        if line.strip() == "## Unreleased":
            insert_at = index + 1
            while insert_at < len(lines) and not lines[insert_at].startswith("## "):
                insert_at += 1
            break
    section = render_version_section(version, today, whats_new, subjects)
    lines = lines[:insert_at] + section + lines[insert_at:]
    return "\n".join(lines).rstrip() + "\n"


def extract_whats_new(text: str, version: str) -> str:
    lines = text.splitlines()
    header = re.compile(rf"^## v{re.escape(version)} - \d{{4}}-\d{{2}}-\d{{2}}$")
    start = None
    for index, line in enumerate(lines):
        if header.match(line.strip()):
            start = index + 1
            break
    if start is None:
        return ""
    section: list[str] = []
    for line in lines[start:]:
        if line.startswith("## "):
            break
        section.append(line)
    in_new = False
    found = False
    body: list[str] = []
    for line in section:
        title = _heading_title(line)
        if title is not None and line.strip().startswith("###"):
            if _is_whats_new(title):
                in_new = True
                found = True
                continue
            if in_new:
                break
        if in_new:
            body.append(line)
    if not found:
        return ""
    notes = _take(body, from_whats_new=True)
    if not notes:
        return ""
    rendered = ["### What's new", ""]
    rendered.extend(f"- {note}" for note in notes)
    rendered.append("")
    return "\n".join(rendered)


def _commit_subjects(version: str) -> list[str]:
    tags = (
        subprocess.check_output(
            ["git", "tag", "--list", "v*", "--sort=-v:refname"], text=True
        )
        .strip()
        .splitlines()
    )
    current = f"v{version}"
    previous = next((tag for tag in tags if tag != current), None)
    range_spec = "HEAD" if previous is None else f"{previous}..HEAD"
    commits = (
        subprocess.check_output(
            ["git", "log", "--no-merges", "--pretty=format:%s", range_spec],
            text=True,
        )
        .strip()
        .splitlines()
    )
    subjects = []
    for subject in commits:
        subject = subject.strip()
        if not subject or "bump version to v" in subject.lower():
            continue
        subjects.append(subject)
    return subjects


def cmd_changelog() -> int:
    version = os.environ["VERSION"]
    today = datetime.date.today().isoformat()
    changelog = Path("CHANGELOG.md")
    if not changelog.exists():
        changelog.write_text(
            "\n".join(
                [
                    "# Changelog",
                    "",
                    "All notable changes to this project will be documented in this file.",
                    "This file is auto-updated by the release workflow.",
                    "",
                    "## Unreleased",
                    "",
                    "- Work in progress.",
                    "",
                ]
            ),
            encoding="utf-8",
        )
    text = changelog.read_text(encoding="utf-8")
    if re.search(
        rf"^## v{re.escape(version)} - \d{{4}}-\d{{2}}-\d{{2}}$",
        text,
        re.M,
    ):
        print(f"CHANGELOG already has v{version}")
        return 0
    updated = update_changelog_text(text, version, today, _commit_subjects(version))
    changelog.write_text(updated, encoding="utf-8")
    return 0


def cmd_body(argv: list[str]) -> int:
    version = os.environ["VERSION"]
    out = "release-whats-new.md"
    if "--out" in argv:
        out = argv[argv.index("--out") + 1]
    changelog = Path("CHANGELOG.md")
    text = changelog.read_text(encoding="utf-8") if changelog.exists() else ""
    Path(out).write_text(extract_whats_new(text, version), encoding="utf-8")
    return 0


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv if argv is None else argv)
    if len(args) < 2 or args[1] not in {"changelog", "body"}:
        print("usage: release_notes.py changelog|body [--out PATH]", file=sys.stderr)
        return 2
    if args[1] == "changelog":
        return cmd_changelog()
    return cmd_body(args[2:])


if __name__ == "__main__":
    raise SystemExit(main())
