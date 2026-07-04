---
icon: lucide/triangle-alert
---

# Error messages

This page lists common errors and their likely fixes.

Start with the first line of the error message, then match it against the table below.

| Error starts with | Likely cause | Fix |
| --- | --- | --- |
| `input file does not exist` | The input path is wrong. | Check the path and run again. |
| `input path is a directory` | The input is a directory, not a file. | Pass a Markdown file. |
| `could not find Chrome, Chromium, or Edge` | Browser discovery failed. | Pass `--browser /path/to/chrome` or set `MD_TO_PDF_BROWSER`. |
| `failed to read Mermaid script` | `--mermaid-js` points to a missing file. | Check the bundle path. |
| `Mermaid render failed` | Mermaid syntax or runtime loading failed. | Fix the diagram or use a local Mermaid bundle. |
| `timed out waiting for Mermaid rendering` | Rendering took longer than the budget. | Increase `--virtual-time-budget`. |
| Local image or asset is missing from the PDF | Browser local file access is disabled or the relative path is wrong. | Use a path relative to the Markdown file and pass `--allow-local-files` for trusted documents. |
