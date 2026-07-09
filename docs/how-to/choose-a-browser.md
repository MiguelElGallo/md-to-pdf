---
icon: lucide/monitor
---

# Choose a browser

This guide shows you how to control which Chromium-family browser renders the PDF.

## Use automatic discovery

Run the command without `--browser`:

```sh
md-to-pdf fixtures/basic.md
```

The CLI looks for Chrome, Chromium, and Microsoft Edge command names on `PATH`, then checks common macOS application paths.

## Set a browser path for one command

Pass `--browser`:

```sh
md-to-pdf fixtures/basic.md \
  --browser "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"
```

Replace the path with the browser executable on your system:

| Platform | Example browser path |
| --- | --- |
| macOS Edge | `/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge` |
| Linux Chrome | `/usr/bin/google-chrome` |
| Windows Chrome | `C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe` |

## Set a browser path for a shell session

Set `MD_TO_PDF_BROWSER`:

```sh
export MD_TO_PDF_BROWSER="/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"
md-to-pdf fixtures/basic.md
```

Use this form in CI when the browser path is known.
