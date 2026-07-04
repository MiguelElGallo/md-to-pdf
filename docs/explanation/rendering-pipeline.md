---
icon: lucide/workflow
---

# Rendering pipeline

Why does conversion go through HTML and a browser?

`md-to-pdf` uses an HTML-first rendering pipeline because Markdown, Mermaid, CSS, and PDF layout all meet naturally in a browser.

For this command:

```sh
md-to-pdf guide.md --output guide.pdf
```

the pipeline is:

1. Parse Markdown with `pulldown-cmark`.
2. Rewrite fenced Mermaid code blocks into Mermaid containers.
3. Generate a print-focused HTML document.
4. Launch Chrome, Chromium, or Edge with the Chrome DevTools Protocol.
5. Wait for Mermaid to report `ready`, or fail on `error`.
6. Ask the browser for PDF bytes with `Page.printToPDF`.

The Rust code owns orchestration, paths, escaping, and CLI behavior. The browser owns JavaScript execution, diagram layout, CSS layout, and PDF generation.

Related tasks: [Debug rendering](../how-to/debug-rendering.md) and [Choose a browser](../how-to/choose-a-browser.md).
