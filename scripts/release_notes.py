"""Build GitHub release notes from the matching CHANGELOG.md section."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

FOOTER = """
### macOS trust status

{macos_status}

### Verification

Download the archive and matching `.sha256` file, then run `shasum -a 256 -c <file>.sha256` on macOS/Linux or `Get-FileHash` on Windows.
"""


def changelog_section(changelog: str, version: str) -> str:
    """Return the body of the `## [version]` section, or raise LookupError."""
    heading = re.compile(rf"^## \[{re.escape(version)}\][^\n]*\n", re.MULTILINE)
    match = heading.search(changelog)
    if match is None:
        raise LookupError(f"CHANGELOG.md has no '## [{version}]' section")
    next_heading = re.search(r"^## ", changelog[match.end() :], re.MULTILINE)
    end = match.end() + next_heading.start() if next_heading else len(changelog)
    body = changelog[match.end() : end].strip()
    if not body:
        raise LookupError(f"CHANGELOG.md section '## [{version}]' is empty")
    return body


def release_notes(changelog: str, version: str, tag: str, macos_status: str) -> str:
    return (
        f"## md-to-pdf {tag}\n\n"
        "This release includes platform archives, SHA-256 checksum files, "
        "and GitHub artifact attestations.\n\n"
        f"{changelog_section(changelog, version)}\n"
        f"{FOOTER.format(macos_status=macos_status)}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="release version without the leading v")
    parser.add_argument("--changelog", type=Path, default=Path("CHANGELOG.md"))
    parser.add_argument("--tag", help="release tag; defaults to v<version>")
    parser.add_argument("--macos-status", help="macOS trust status paragraph")
    parser.add_argument("--output", type=Path, help="write notes to this file")
    args = parser.parse_args()

    changelog = args.changelog.read_text(encoding="utf-8")
    try:
        if args.macos_status is None:
            changelog_section(changelog, args.version)
            return 0
        notes = release_notes(
            changelog, args.version, args.tag or f"v{args.version}", args.macos_status
        )
    except LookupError as error:
        print(error, file=sys.stderr)
        return 1

    if args.output:
        args.output.write_text(notes, encoding="utf-8")
    else:
        sys.stdout.write(notes)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
