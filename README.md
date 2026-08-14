# md-to-pdf

[![CI](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/ci.yml)
[![Documentation](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/docs.yml/badge.svg)](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/docs.yml)
[![Release](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/release.yml/badge.svg)](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/release.yml)
[![Latest release](https://img.shields.io/github/v/release/MiguelElGallo/md-to-pdf)](https://github.com/MiguelElGallo/md-to-pdf/releases/latest)

Convert one Markdown file into one PDF, with Mermaid diagrams rendered before the PDF is written.

`md-to-pdf` reads one `.md` file, renders GitHub-style Markdown to browser-ready HTML, waits for Mermaid diagrams to finish, and writes a PDF using Chrome, Chromium, or Edge.

Full documentation is published at <https://miguelelgallo.github.io/md-to-pdf/> and organized with [Diataxis](https://diataxis.fr/): tutorials, how-to guides, reference, and explanation. The site is built with [Zensical](https://zensical.org/), and the Markdown sources live in [docs/index.md](docs/index.md).

## Requirements

- Rust with Cargo.
- Chrome, Chromium, or Microsoft Edge.
- Internet access for Mermaid diagrams by default.

Plain Markdown conversion does not need network access. Mermaid diagrams use jsDelivr by default, but you can pass a local Mermaid browser bundle for offline or reproducible builds.

## Install

macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | sh
```

The installer detects Apple Silicon vs Intel, downloads the latest macOS archive and matching checksum, verifies the checksum, and installs `md-to-pdf` to `/usr/local/bin`.

To install a specific release instead of the latest one, set `MD_TO_PDF_VERSION`:

```sh
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | MD_TO_PDF_VERSION=v0.1.3 sh
```

Linux:

```sh
VERSION="vX.Y.Z" # replace with the tag of the release you downloaded
shasum -a 256 -c "md-to-pdf-${VERSION}-x86_64-unknown-linux-gnu.sha256"
tar -xzf "md-to-pdf-${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
sudo install "md-to-pdf-${VERSION}-x86_64-unknown-linux-gnu/md-to-pdf" /usr/local/bin/md-to-pdf
md-to-pdf --help
```

Windows PowerShell:

```powershell
$version = "vX.Y.Z" # replace with the tag of the release you downloaded
$archive = ".\md-to-pdf-$version-x86_64-pc-windows-msvc.zip"
Get-FileHash $archive -Algorithm SHA256
Expand-Archive $archive
.\md-to-pdf-$version-x86_64-pc-windows-msvc\md-to-pdf.exe --help
```

Compare the hash with the matching `.sha256` file before running the binary.

Install from source for development:

```sh
cargo install --path .
```

For development:

```sh
cargo build
```

## Quickstart

Create a file named `example.md`:

````markdown
# Release Flow

This diagram will render inside the PDF.

```mermaid
graph TD
  A[Write Markdown] --> B[Render Mermaid]
  B --> C[Print PDF]
```
````

Convert it:

```sh
md-to-pdf example.md
```

You should see:

```text
Wrote example.pdf
```

The default output path replaces the input file extension with `.pdf`.
Use `--title "Release Flow"` to set the document title stored in the generated HTML and PDF metadata; by default, it uses the input file name without its extension.

## Agent Plugin (MCP)

`md-to-pdf` ships as an **[Agent Plugins 1.0.0](https://github.com/agentplugins/agent-plugins-spec)**-compliant plugin. Any agent client that supports the spec can load it directly.

### Plugin layout

```
md-to-pdf/
├── plugin.json                  # Agent Plugins manifest
├── mcp.json                     # MCP stdio server config
├── mcp_server/
│   └── server.py                # MCP stdio server (Python 3 stdlib, no extra deps)
└── skills/
    └── convert-to-pdf/
        └── SKILL.md             # Agent Skill definition
```

### Install the plugin

**1. Clone or download the repo** (the plugin root is the repo root):

```sh
git clone https://github.com/MiguelElGallo/md-to-pdf
```

**2. Install the `md-to-pdf` binary** (required by the MCP server at runtime):

```sh
# macOS
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | sh

# or build from source
cargo install --path md-to-pdf
```

**3. Install a supported browser** (Chrome, Chromium, or Edge) if not already present.

**4. Register the plugin** with your agent client by pointing it at the plugin root. For example, in a client that reads `mcp.json` directly:

```json
{
  "plugins": [
    { "path": "/path/to/md-to-pdf" }
  ]
}
```

The client reads `plugin.json`, discovers the skill in `skills/convert-to-pdf/SKILL.md`, and launches the MCP server defined in `mcp.json` as:

```sh
python3 /path/to/md-to-pdf/mcp_server/server.py
```

> No Python packages are required — `server.py` uses only the standard library.

### Use the MCP tool directly

You can also talk to the MCP server manually over stdio (useful for testing):

```sh
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | python3 mcp_server/server.py
```

#### Available tool: `convert_markdown_to_pdf`

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `input` | string | ✅ | — | Path to the Markdown file |
| `output` | string | | input + `.pdf` | Output PDF path |
| `title` | string | | filename stem | PDF metadata title |
| `page_size` | string | | `A4` | CSS page size (`A4`, `Letter`, `Legal`, …) |
| `css` | string | | — | Extra CSS file to append after built-in styles |
| `mermaid_url` | string | | jsDelivr CDN | Mermaid ES module URL |
| `allow_html` | boolean | | `false` | Pass raw HTML through |
| `allow_local_files` | boolean | | `false` | Allow Chrome local file access |
| `browser` | string | | `MD_TO_PDF_BROWSER` env | Browser executable path |

Example tool call:

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "convert_markdown_to_pdf",
    "arguments": {
      "input": "/path/to/document.md",
      "output": "/path/to/document.pdf",
      "page_size": "Letter"
    }
  }
}
```

On success the tool returns the path to the generated PDF:

```json
{
  "content": [{ "type": "text", "text": "/path/to/document.pdf" }]
}
```

### Environment variables

| Variable | Description |
|----------|-------------|
| `MD_TO_PDF_BROWSER` | Path to Chrome/Chromium/Edge executable |
| `MD_TO_PDF_BIN` | Path to `md-to-pdf` binary (overrides PATH lookup) |



- Follow the guided docs in [docs/index.md](docs/index.md).
- Look up flags and defaults in [docs/reference/cli.md](docs/reference/cli.md).
- Render Mermaid offline with [docs/how-to/use-local-mermaid.md](docs/how-to/use-local-mermaid.md).
- Choose a browser with [docs/how-to/choose-a-browser.md](docs/how-to/choose-a-browser.md).
- Inspect rendering problems with [docs/how-to/debug-rendering.md](docs/how-to/debug-rendering.md).
- Understand safety defaults in [docs/explanation/safety-model.md](docs/explanation/safety-model.md).
- Read the browser pipeline rationale in [docs/explanation/rendering-pipeline.md](docs/explanation/rendering-pipeline.md).

## Development

Run the standard checks:

```sh
cargo fmt --check
cargo test
uv run --locked --group docs zensical build --clean --strict
```

Run browser smoke tests by setting `MD_TO_PDF_BROWSER`:

```sh
MD_TO_PDF_BROWSER="/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge" \
  cargo test browser_smoke -- --nocapture
```

Manual smoke checks:

```sh
cargo run -- fixtures/basic.md --output /tmp/basic.pdf
cargo run -- fixtures/mermaid-flowchart.md --output /tmp/mermaid.pdf --virtual-time-budget 15000
cargo run -- fixtures/invalid-mermaid.md --output /tmp/invalid.pdf --virtual-time-budget 15000
```

The first two commands should create nonempty PDFs. The invalid Mermaid fixture should fail with `Mermaid render failed`.

## Current Scope

Included in this MVP:

- Single Markdown file to single PDF.
- Headings, lists, tables, task lists, code blocks, links, images, and basic GitHub-style Markdown extensions from `pulldown-cmark`.
- Mermaid fenced blocks rendered by a browser.
- Custom output path, page size, CSS, browser path, local Mermaid script, and generated HTML debugging.

Deferred for later releases:

- Batch conversion and glob support.
- Watch mode.
- Headers, footers, page numbers, and table of contents.
- Pixel-perfect PDF regression testing.
- Bundled Chromium or bundled Mermaid assets.
- Strong sandbox guarantees for untrusted Markdown.

## Release Checklist

- `cargo fmt --check` passes.
- `cargo test` passes.
- Browser smoke tests pass for plain Markdown, valid Mermaid, and invalid Mermaid.
- README quickstart is verified from a fresh clone.
- macOS, Linux, and Windows browser discovery are checked or documented.
- Offline Mermaid flow with `--mermaid-js` is verified before publishing a reproducible release.
- Release artifacts are extracted, checksum-verified, and smoke-tested before publishing.

GitHub Actions includes CI, documentation, and release workflows inspired by Astral's `uv` release setup: strict default permissions, concurrency control, an aggregate required-checks job, Zensical docs builds, multi-platform release artifacts, and SHA-256 checksum files.

Run a release dry run from the Actions tab with the `Release` workflow and `tag=dry-run`. It builds all archives and uploads them as workflow artifacts without creating a GitHub Release.

Publish a release by pushing a SemVer tag:

```sh
VERSION=vX.Y.Z # replace with the new package version
git tag "$VERSION"
git push origin "$VERSION"
```

You can also manually dispatch the `Release` workflow with a tag such as `vX.Y.Z`. The release workflow validates that the tag matches the package version before publishing.

## Roadmap

1. Add Apple Developer ID signing and notarization with `.zip`, `.pkg`, or `.dmg` macOS artifacts for trusted downloads.
2. Add browser smoke coverage on macOS and Windows.
3. Add PDF inspection tests for page count and expected text.
4. Add `--out-dir` and multiple input support.
5. Add config/front matter for page size, margins, theme, and Mermaid settings.
