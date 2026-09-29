# Changelog

All notable changes to md-to-pdf are documented here. The release workflow publishes the section matching the tag as the GitHub release notes, and refuses to publish a tag without one.

## [0.7.1]

### Fixed

- Hard-linked PDF and debug HTML destinations can no longer overwrite the input Markdown or collide with one another.
- PDF and HTML output use same-directory atomic replacement. Partial writes and failed replacements leave existing destinations unchanged and clean up temporary files; unrelated hard-linked backups remain untouched.
- Output validation rejects symlinks, directories, special files, and read-only files before rendering, without blocking on named pipes.
- Atomic replacement preserves ordinary file permission bits and rejects ownership, ACL, security-descriptor, or Windows encryption changes that could weaken file protection.
- The existing `--virtual-time-budget` option is documented accurately as a per-attempt wall-clock readiness timeout, not Chromium virtual time or an end-to-end deadline.

### Upgrade notes

Use direct, writable output-file paths rather than symlinks. Unix outputs with extended ACLs, mismatched ownership, or special permission bits, and encrypted or security-incompatible Windows outputs, are rejected instead of silently losing their protection. New Unix output files request mode `0600`; Windows files use inherited security. Parent-directory symlinks remain supported. Debug HTML is retained when rendering fails and is not committed as a transaction with the PDF.

## [0.7.0]

### Added

- Transient Mermaid download failures are retried automatically. When the Mermaid runtime or one of its chunks fails to load, md-to-pdf prints a message to stderr and reloads the page after 1, 3, 5, and then 10 seconds, for up to five attempts. Diagram syntax errors are still reported immediately.
- The MCP server writes the converter's diagnostics, such as retry messages, to its log, and scales its conversion timeout so every retry fits within it.

## [0.6.1]

### Fixed

- Converting a Markdown file passed as a bare file name, such as `md-to-pdf report.md` or the MCP input `report.md`, no longer fails with `failed to resolve`. The file's directory now defaults to the current directory.

## [0.6.0]

### Added

- Headings get GitHub-style anchor ids, so in-document links such as `[Results](#results)` are clickable in the PDF. Explicit `{#id}` attributes are kept, and duplicates get `-1`, `-2`, and so on.
- PDFs include a document outline (bookmarks) built from headings and are generated as tagged PDFs for better accessibility.
- `--page-size` accepts CSS forms such as `A4 landscape` and `210mm 297mm`.

### Fixed

- `--page-size` (and the MCP `page_size` option) is validated, and values containing characters that could inject CSS into the `@page` rule are rejected. The library API uses a validated `PageSize` type, so every entry point is covered.
- Heading anchors no longer collide with footnote ids.

### Upgrade notes

Page sizes containing characters other than letters, digits, spaces, `.`, and `-` are now rejected with `invalid page size`.

## [0.5.1]

### Changed

- Blocked HTTP(S) assets referenced by Markdown, CSS, or raw HTML by default. Trusted documents can explicitly restore them with `--allow-remote-assets` or the matching MCP option; the built-in Mermaid CDN remains available.
- Removed workflow-dispatch command injection from the release planner by validating tag input before any signing secret is available.
- Pinned every release action to an immutable commit and isolated macOS signing/notarization onto fresh runners after secret-free builds.
- Added browser-level SSRF regression tests, release-workflow structure tests, and macOS code-signing and notarization verification.

### Upgrade notes

Remote document assets that previously loaded automatically now require `allow_remote_assets: true` (MCP) or `--allow-remote-assets` (CLI). This trusted-content opt-in permits requests to private and local networks. A custom Mermaid URL also requires the opt-in; the default Mermaid CDN and local `mermaid_js` bundles continue to work without it.
