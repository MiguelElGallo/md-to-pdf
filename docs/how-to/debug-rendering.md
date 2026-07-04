---
icon: lucide/bug
---

# Debug rendering

This guide shows you how to inspect the generated HTML when the PDF output is not what you expect.

## Keep the HTML file

Run:

```sh
md-to-pdf fixtures/mermaid-flowchart.md --keep-html
```

The CLI writes `fixtures/mermaid-flowchart.html` next to the PDF.

Open that HTML file in the same browser you use for conversion. If the HTML looks right but the PDF does not, focus on print CSS and page size. If the HTML is wrong, inspect the Markdown, Mermaid, or asset paths first.

Check the HTML in this order:

- The Markdown content appears.
- Mermaid diagrams are rendered.
- Images and local assets load.
- Print CSS changes appear before converting again.

## Give diagrams more time

If Mermaid diagrams are large or the network is slow, run:

```sh
md-to-pdf fixtures/mermaid-flowchart.md --virtual-time-budget 20000
```

## Check Mermaid syntax

If the command fails with `Mermaid render failed`, reduce the document to the smallest diagram that still fails and fix that diagram first.
