---
icon: lucide/terminal
---

# CLI options

`md-to-pdf` accepts one required input path and the options below.

Common commands:

```sh
md-to-pdf guide.md                                      # default output path
md-to-pdf guide.md --output dist/guide.pdf             # custom output path
md-to-pdf guide.md --title "Project guide"              # custom document title
md-to-pdf guide.md --css print.css                     # extra print CSS
md-to-pdf guide.md --mermaid-js ./vendor/mermaid.min.js # local Mermaid bundle
```

| Option | Default | Description |
| --- | --- | --- |
| `input` | Required | Markdown file to convert. |
| `-o, --output <PATH>` | `<input>.pdf` | PDF output path. |
| `--title <TITLE>` | Input file name without extension | Document title stored in the generated HTML and PDF metadata. |
| `--browser <PATH>` | Auto-detect or `MD_TO_PDF_BROWSER` | Chrome, Chromium, or Edge executable to use. |
| `--page-size <SIZE>` | `A4` | CSS page size such as `A4`, `Letter`, or `Legal`. |
| `--css <PATH>` | None | Extra CSS file appended after built-in print styles. |
| `--mermaid-url <URL>` | jsDelivr Mermaid 11.12.0 | Mermaid ES module URL. Conflicts with `--mermaid-js`. |
| `--mermaid-js <PATH>` | None | Local Mermaid browser bundle that exposes `window.mermaid`. Conflicts with `--mermaid-url`. |
| `--allow-html` | `false` | Let raw HTML in Markdown pass through. |
| `--allow-local-files` | `false` | Pass Chrome's `--allow-file-access-from-files` flag. |
| `--virtual-time-budget <MS>` | `10000` | Milliseconds to wait for page load and Mermaid rendering. |
| `--keep-html` | `false` | Write generated HTML next to the PDF. |

The output path cannot be the input Markdown file. When `--keep-html` is used, choose a PDF output path that does not use an `.html` extension, so the debug HTML and PDF have distinct paths.

Run `md-to-pdf --help` for the executable's current help text.
