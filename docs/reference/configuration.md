---
icon: lucide/settings
---

# Configuration

`md-to-pdf` is configured with CLI options and one environment variable.

## Environment variables

| Variable | Description |
| --- | --- |
| `MD_TO_PDF_BROWSER` | Browser executable path used when `--browser` is not passed. |

## Browser discovery

Browser selection uses this precedence:

1. `--browser`
2. `MD_TO_PDF_BROWSER`
3. automatic discovery

Automatic discovery checks common Chrome, Chromium, and Edge command names, then common macOS application paths.

On systems where discovery fails, pass `--browser` or set `MD_TO_PDF_BROWSER`.

```sh
MD_TO_PDF_BROWSER="/usr/bin/google-chrome" md-to-pdf guide.md
```

## Output path

If `--output` is not passed, the output path replaces the input extension with `.pdf`. The PDF output cannot be the input file; when `--keep-html` is used, the output path also cannot use an `.html` extension.
