# Error messages

This page lists common errors and their likely fixes.

| Error starts with | Likely cause | Fix |
| --- | --- | --- |
| `input file does not exist` | The input path is wrong. | Check the path and run again. |
| `input path is a directory` | The input is a directory, not a file. | Pass a Markdown file. |
| `could not find Chrome, Chromium, or Edge` | Browser discovery failed. | Pass `--browser` or set `MD_TO_PDF_BROWSER`. |
| `failed to read Mermaid script` | `--mermaid-js` points to a missing file. | Check the bundle path. |
| `Mermaid render failed` | Mermaid syntax or runtime loading failed. | Fix the diagram or use a local Mermaid bundle. |
| `timed out waiting for Mermaid rendering` | Rendering took longer than the budget. | Increase `--virtual-time-budget`. |
