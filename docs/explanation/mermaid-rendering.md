# Mermaid rendering

Mermaid is JavaScript-native, so `md-to-pdf` renders Mermaid in the browser instead of trying to reimplement diagram layout in Rust.

By default, the generated HTML imports Mermaid from jsDelivr. This is convenient for first use, but it depends on network access and the hosted Mermaid version.

For reproducible output, use a local Mermaid browser bundle with `--mermaid-js`. That makes the Mermaid runtime part of your build inputs.

The browser page records Mermaid status in `data-mermaid-status`. The CLI waits for `ready`, fails on `error`, and times out if rendering does not finish in the virtual time budget.
