# md-to-pdf

[![CI](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/MiguelElGallo/md-to-pdf/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/MiguelElGallo/md-to-pdf)](https://github.com/MiguelElGallo/md-to-pdf/releases/latest)

Convert Markdown to PDF with Mermaid diagrams using Chrome, Chromium, or Edge.

## Install as an Agent Plugin

This repository is an [Agent Plugins 1.0.0](https://github.com/agentplugins/agent-plugins-spec)-compliant package.

1. Install Python 3 and Chrome, Chromium, or Edge.
2. Install the `md-to-pdf` binary:

   ```sh
   # macOS
   curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | sh

   # Or build from source
   cargo install --git https://github.com/MiguelElGallo/md-to-pdf
   ```

3. Clone the plugin:

   ```sh
   git clone https://github.com/MiguelElGallo/md-to-pdf
   ```

4. In an Agent Plugins-compatible client, add the cloned repository as the plugin root.

The client reads `plugin.json`, discovers `skills/convert-to-pdf/SKILL.md`, and launches the MCP server from `mcp.json`. The server uses only the Python standard library.

### MCP tool

`convert_markdown_to_pdf` requires `input` and supports `output`, `title`, `page_size`, `css`, `mermaid_url`, `allow_html`, `allow_local_files`, and `browser`.

Example arguments:

```json
{
  "input": "/path/to/document.md",
  "output": "/path/to/document.pdf",
  "page_size": "Letter"
}
```

Set `MD_TO_PDF_BIN` to override the binary path or `MD_TO_PDF_BROWSER` to select a browser.

## Install the CLI

Download a release from [GitHub Releases](https://github.com/MiguelElGallo/md-to-pdf/releases/latest), or install on macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | sh
```

Build from source:

```sh
cargo install --git https://github.com/MiguelElGallo/md-to-pdf
```

## Use the CLI

```sh
md-to-pdf document.md
```

The default output is `document.pdf`. Common options:

```sh
md-to-pdf document.md \
  --output report.pdf \
  --title "Report" \
  --page-size Letter \
  --css print.css
```

Mermaid fenced code blocks render automatically:

````markdown
```mermaid
graph TD
  A[Markdown] --> B[PDF]
```
````

Use `md-to-pdf --help` for all options.

## Requirements

- Chrome, Chromium, or Microsoft Edge
- Internet access for Mermaid by default; use `--mermaid-js` for offline rendering
- Rust and Cargo only when building from source

## Documentation

- [Full documentation](https://miguelelgallo.github.io/md-to-pdf/)
- [CLI reference](docs/reference/cli.md)
- [Choose a browser](docs/how-to/choose-a-browser.md)
- [Use Mermaid offline](docs/how-to/use-local-mermaid.md)
- [Safety model](docs/explanation/safety-model.md)

## Development

```sh
cargo fmt --check
cargo test
uv run --locked --group docs zensical build --clean --strict
```

See [LICENSE](LICENSE).
