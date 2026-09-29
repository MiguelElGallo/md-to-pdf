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
| `--page-size <SIZE>` | `A4` | CSS page size such as `A4`, `Letter`, `Legal`, `A4 landscape`, or `210mm 297mm`. Only letters, digits, spaces, `.`, and `-` are accepted. |
| `--css <PATH>` | None | Extra CSS file appended after built-in print styles. |
| `--mermaid-url <URL>` | jsDelivr Mermaid 11.12.0 | Mermaid ES module URL. Conflicts with `--mermaid-js`. |
| `--mermaid-js <PATH>` | None | Local Mermaid browser bundle that exposes `window.mermaid`. Conflicts with `--mermaid-url`. |
| `--allow-html` | `false` | Let raw HTML in Markdown pass through. |
| `--allow-local-files` | `false` | Pass Chrome's `--allow-file-access-from-files` flag. |
| `--allow-remote-assets` | `false` | Allow HTTP(S) assets from Markdown, CSS, or trusted raw HTML. This can reach private and local networks. |
| `--virtual-time-budget <MS>` | `10000` | Per-attempt wall-clock timeout for page load and Mermaid readiness, in milliseconds. Not Chromium virtual time; browser startup and retry delays are separate, and browser commands can overrun this timeout. |
| `--keep-html` | `false` | Write generated HTML next to the PDF. |

The output path cannot be the input Markdown file, including a hard-link alias. When `--keep-html` is used, choose a PDF output path that does not use an `.html` extension. The debug HTML cannot be an alias of the input or PDF output either.

Output destinations must be direct, writable regular-file paths, not symlinks (including dangling symlinks), directories, or special files. Parent-directory symlinks are permitted. PDF and HTML files are replaced atomically using a temporary file in the destination directory. A failed write leaves any existing destination unchanged. Existing permission bits are preserved. Replacement is rejected if ownership or access-control restrictions cannot be preserved: Unix destinations with extended ACLs or special permission bits are rejected, and Windows destinations must be unencrypted and have matching security descriptors. New files request mode `0600` on Unix and use default inherited security on Windows. Replacement changes the destination's file identity and leaves other hard-linked aliases unchanged. This assumes a trusted output directory without concurrent security-metadata changes.

With `--keep-html`, the generated HTML is retained even when browser rendering fails, so it can be inspected. HTML and PDF replacement are separate operations, not a combined transaction.

Transient Mermaid download failures can trigger up to four retries, each with its own readiness timeout, separated by delays of 1, 3, 5, and 10 seconds. A browser command can take up to its separate 30-second timeout, so `--virtual-time-budget` is not a strict end-to-end time limit.

Run `md-to-pdf --help` for the executable's current help text.
