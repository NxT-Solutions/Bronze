#!/usr/bin/env python3
import unittest

from release_notes import (
    NOTE_CHECK_FOR_UPDATES,
    NOTE_ENGINE_SWITCH,
    NOTE_PERMISSIONS,
    extract_whats_new,
    release_notes_from_markdown,
    update_changelog_text,
)

V012 = "\n".join(
    [
        "## What's Changed",
        "* fix(desktop): style Check for updates as a push button by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/44",
        "* fix(release): bleed the Dock mark to the icon canvas by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/45",
        "* fix(title): settle on-this-Mac engine switches by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/46",
        "* feat(desktop): prompt used macOS permissions once by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/49",
        "* chore(release): set version 0.1.2 by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/47",
        "* fix(storage): retune sqlite wal checkpoint pragma by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/99",
        "",
        "**Full Changelog**: https://github.com/NxT-Solutions/Bronze/compare/v0.1.1...v0.1.2",
    ]
)


class ReleaseNotesTest(unittest.TestCase):
    def test_conventional_commit_becomes_a_plain_bullet(self):
        notes = release_notes_from_markdown(
            "* fix(desktop): style Check for updates as a push button by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/44"
        )
        self.assertEqual(notes, [NOTE_CHECK_FOR_UPDATES])
        self.assertNotIn("http", notes[0])
        self.assertNotIn("@", notes[0])

    def test_unknown_jargon_is_omitted_and_urls_are_not_the_wrap(self):
        notes = release_notes_from_markdown(V012)
        self.assertEqual(
            notes,
            [
                NOTE_CHECK_FOR_UPDATES,
                NOTE_ENGINE_SWITCH,
                NOTE_PERMISSIONS,
            ],
        )
        for note in notes:
            self.assertNotIn("http", note)
            self.assertNotIn("@", note)
            self.assertNotIn("fix(", note)
            self.assertNotIn("Dock", note)
            self.assertNotIn("tile", note)
            self.assertGreaterEqual(len(note.split()), 2)

    def test_whats_new_section_is_preferred(self):
        notes = release_notes_from_markdown(
            "### What's new\n\n- Search is faster.\n\n## What's Changed\n"
            "* fix(storage): retune sqlite wal checkpoint pragma by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/9\n"
        )
        self.assertEqual(notes, ["Search is faster."])

    def test_version_line_is_unchanged_and_not_a_bullet(self):
        notes = release_notes_from_markdown(
            "Version 0.1.2 is available.\n### What's new\n- Check for updates is easier to see.\n"
        )
        self.assertEqual(notes, [NOTE_CHECK_FOR_UPDATES])
        self.assertTrue(all("Version" not in note for note in notes))

    def test_existing_version_section_is_not_rewritten(self):
        original = "\n".join(
            [
                "# Changelog",
                "",
                "## Unreleased",
                "",
                "- Work in progress.",
                "",
                "## v0.1.2 - 2026-09-25",
                "",
                "- The Dock icon fills the tile.",
                "",
            ]
        )
        text = original if original.endswith("\n") else original + "\n"
        updated = update_changelog_text(
            text,
            "0.1.2",
            "2026-09-26",
            ["fix(desktop): style Check for updates as a push button"],
        )
        self.assertEqual(updated, text)
        self.assertEqual(extract_whats_new(updated, "0.1.2"), "")

    def test_generator_writes_plain_whats_new_and_keeps_commits(self):
        original = "\n".join(
            [
                "# Changelog",
                "",
                "## Unreleased",
                "",
                "- Work in progress.",
                "",
            ]
        )
        subjects = [
            "fix(desktop): style Check for updates as a push button by @NoahNxT in https://github.com/NxT-Solutions/Bronze/pull/44",
            "fix(storage): retune sqlite wal checkpoint pragma",
        ]
        updated = update_changelog_text(original + "\n", "0.2.0", "2026-09-26", subjects)
        self.assertIn("### What's new", updated)
        self.assertIn(f"- {NOTE_CHECK_FOR_UPDATES}", updated)
        self.assertNotIn("retune sqlite", updated.split("### Changes")[0])
        self.assertIn("retune sqlite", updated.split("### Changes")[1])
        self.assertNotIn("@NoahNxT", updated.split("### What's new")[1].split("### Changes")[0])
        body = extract_whats_new(updated, "0.2.0")
        self.assertIn(NOTE_CHECK_FOR_UPDATES, body)
        self.assertNotIn("http", body)
        self.assertNotIn("### Changes", body)

    def test_human_unreleased_lines_become_whats_new(self):
        original = "\n".join(
            [
                "# Changelog",
                "",
                "## Unreleased",
                "",
                "- Search is faster.",
                "",
                "## v0.1.2 - 2026-09-25",
                "",
                "- The Dock icon fills the tile.",
                "",
            ]
        )
        updated = update_changelog_text(
            original + "\n",
            "0.2.0",
            "2026-09-26",
            ["fix(storage): retune sqlite wal checkpoint pragma"],
        )
        whats_new = updated.split("### What's new", 1)[1].split("### Changes", 1)[0]
        self.assertIn("- Search is faster.", whats_new)
        self.assertNotIn("retune sqlite", whats_new)
        self.assertIn("- Work in progress.", updated.split("## v0.2.0", 1)[0])


if __name__ == "__main__":
    unittest.main()
