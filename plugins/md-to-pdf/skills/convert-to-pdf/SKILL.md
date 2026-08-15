---
name: convert-to-pdf
description: Convert a Markdown file (with optional Mermaid diagrams) to a PDF document using a headless browser renderer.
---

# Convert Markdown to PDF

Convert a Markdown file to a PDF document. Supports Mermaid diagrams, configurable page sizes, custom CSS stylesheets, and PDF metadata.

## Usage

Call the `convert_markdown_to_pdf` MCP tool with the path to a Markdown file. The tool returns the path to the generated PDF file.

## Parameters

- `input` (required): Path to the Markdown file to convert.
- `output` (optional): Output PDF path. Defaults to the input filename with a `.pdf` extension.
- `title` (optional): Document title stored in PDF metadata. Defaults to the input filename without extension.
- `page_size` (optional): CSS page size such as `A4`, `Letter`, or `Legal`. Defaults to `A4`.
- `css` (optional): Path to an extra CSS file to append after the built-in print styles.
- `mermaid_url` (optional): Mermaid ES module URL for diagram rendering. Overrides the default CDN URL.
- `allow_html` (optional): Allow raw HTML in Markdown to pass through. Defaults to `false`.
- `allow_local_files` (optional): Allow Chrome to access local files for assets referenced in Markdown. Defaults to `false`.
- `browser` (optional): Path to Chrome, Chromium, or Edge executable. Falls back to the `MD_TO_PDF_BROWSER` environment variable.

## Examples

Convert a simple Markdown file:

```text
input: /path/to/document.md
```

Convert with a specific output path and Letter page size:

```text
input: /path/to/report.md
output: /path/to/report.pdf
page_size: Letter
```

Convert with a custom stylesheet:

```text
input: /path/to/report.md
css: /path/to/custom.css
```

## Notes

- Mermaid diagrams embedded in `mermaid` fenced code blocks are rendered automatically.
- A Chrome, Chromium, or Edge browser must be available. Set `MD_TO_PDF_BROWSER` to specify the executable path.
- On success, the tool returns the absolute path to the generated PDF file.
