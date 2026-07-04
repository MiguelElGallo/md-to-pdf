---
icon: lucide/network
---

# Use Mermaid offline

This guide shows you how to avoid loading Mermaid from the default CDN.

Use this in CI, offline environments, or reproducible release builds.

## Provide a local Mermaid bundle

Download a Mermaid browser bundle that exposes `window.mermaid`, then run:

```sh
md-to-pdf fixtures/mermaid-flowchart.md --mermaid-js ./vendor/mermaid.min.js
```

The local script is embedded in the generated HTML, so the browser does not need to fetch Mermaid from jsDelivr.

The command should write the PDF without fetching Mermaid from the network.

Use `--mermaid-js` for a local browser script. Use `--mermaid-url` only when you want the generated HTML to load Mermaid from an ES module URL.

## Increase render time for large diagrams

If a large diagram needs more time, increase the virtual time budget:

```sh
md-to-pdf fixtures/mermaid-flowchart.md \
  --mermaid-js ./vendor/mermaid.min.js \
  --virtual-time-budget 20000
```

For CI, combine this with [Run in CI](run-in-ci.md).
