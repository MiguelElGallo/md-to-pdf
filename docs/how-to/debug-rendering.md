# Debug rendering

This guide shows you how to inspect the generated HTML when the PDF output is not what you expect.

## Keep the HTML file

Run:

```sh
md-to-pdf fixtures/mermaid-flowchart.md --keep-html
```

The CLI writes `fixtures/mermaid-flowchart.html` next to the PDF.

## Give diagrams more time

If Mermaid diagrams are large or the network is slow, run:

```sh
md-to-pdf fixtures/mermaid-flowchart.md --virtual-time-budget 20000
```

## Check Mermaid syntax

If the command fails with `Mermaid render failed`, reduce the document to the smallest diagram that still fails and fix that diagram first.
