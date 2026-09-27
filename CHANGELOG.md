# Changelog

All notable changes to md-to-pdf are documented here. The release workflow publishes the section matching the tag as the GitHub release notes, and refuses to publish a tag without one.

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
