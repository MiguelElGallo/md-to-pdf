# Configuration

`md-to-pdf` is configured with CLI options and one environment variable.

## Environment variables

| Variable | Description |
| --- | --- |
| `MD_TO_PDF_BROWSER` | Browser executable path used when `--browser` is not passed. |

## Browser discovery

The CLI checks common Chrome, Chromium, and Edge command names, then common macOS application paths.

On systems where discovery fails, pass `--browser` or set `MD_TO_PDF_BROWSER`.

## Output path

If `--output` is not passed, the output path is the input path with a `.pdf` extension.
