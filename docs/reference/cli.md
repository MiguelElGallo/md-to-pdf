# CLI options

`md-to-pdf` accepts one required input path and the options below.

| Option | Default | Description |
| --- | --- | --- |
| `input` | Required | Markdown file to convert. |
| `-o, --output <PATH>` | `<input>.pdf` | PDF output path. |
| `--browser <PATH>` | Auto-detect or `MD_TO_PDF_BROWSER` | Chrome, Chromium, or Edge executable to use. |
| `--page-size <SIZE>` | `A4` | CSS page size such as `A4`, `Letter`, or `Legal`. |
| `--css <PATH>` | None | Extra CSS file appended after built-in print styles. |
| `--mermaid-url <URL>` | jsDelivr Mermaid 11.12.0 | Mermaid ES module URL. Conflicts with `--mermaid-js`. |
| `--mermaid-js <PATH>` | None | Local Mermaid browser bundle that exposes `window.mermaid`. Conflicts with `--mermaid-url`. |
| `--allow-html` | `false` | Let raw HTML in Markdown pass through. |
| `--allow-local-files` | `false` | Pass Chrome's `--allow-file-access-from-files` flag. |
| `--virtual-time-budget <MS>` | `10000` | Milliseconds to wait for page load and Mermaid rendering. |
| `--keep-html` | `false` | Write generated HTML next to the PDF. |

Run `md-to-pdf --help` for the executable's current help text.
