---
name: convert-to-pdf
description: Convert a Markdown file (with optional Mermaid diagrams) to a PDF document using a headless browser renderer.
---

Convert a Markdown file to a PDF document. Supports Mermaid diagrams, custom CSS, configurable page sizes, and metadata.

## Usage

Call the `convert_markdown_to_pdf` MCP tool with the path to a Markdown file. The tool returns the path to the generated PDF.

## Parameters

- `input` (required): Path to the Markdown file to convert.
- `output` (optional): Output PDF path. Defaults to the input filename with a `.pdf` extension.
- `title` (optional): Document title stored in PDF metadata. Defaults to the input filename without extension.
- `page_size` (optional): CSS page size such as `A4`, `Letter`, or `Legal`. Defaults to `A4`.
- `allow_html` (optional): Allow raw HTML in Markdown to pass through. Defaults to `false`.
- `allow_local_files` (optional): Allow Chrome to access local files for assets referenced in Markdown. Defaults to `false`.

## Examples

Convert a simple Markdown file:

```
input: /path/to/document.md
```

Convert with a specific output path and Letter page size:

```
input: /path/to/report.md
output: /path/to/report.pdf
page_size: Letter
```

## Notes

- Mermaid diagrams embedded in fenced code blocks (` ```mermaid `) are rendered automatically.
- A Chrome, Chromium, or Edge browser must be available on the system. Set the `MD_TO_PDF_BROWSER` environment variable to specify the executable path.
