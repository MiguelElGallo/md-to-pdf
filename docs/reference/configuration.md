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

Automatic discovery first checks common Chrome, Chromium, and Edge command names on `PATH`. It then checks common macOS application paths and, on Windows, standard installation folders under `ProgramFiles`, `ProgramFiles(x86)`, and `LOCALAPPDATA`.

Within each Windows folder, discovery checks `Google/Chrome/Application/chrome.exe`, `Microsoft/Edge/Application/msedge.exe`, and `Chromium/Application/chrome.exe` in that order. A browser installed in one of these locations does not need to be on `PATH`.

On systems where discovery fails, pass `--browser` or set `MD_TO_PDF_BROWSER`.

```sh
MD_TO_PDF_BROWSER="/usr/bin/google-chrome" md-to-pdf guide.md
```

## Output path

If `--output` is not passed, the output path replaces the input extension with `.pdf`. The PDF output cannot be the input file; when `--keep-html` is used, the output path also cannot use an `.html` extension.
