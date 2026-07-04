---
icon: lucide/shield-check
---

# Safety model

What risks do the defaults reduce, and what remains outside scope?

The CLI has conservative defaults, but it is not a complete sandbox for hostile documents.

Raw HTML in Markdown is escaped by default. Passing `--allow-html` changes that behavior and should be reserved for trusted Markdown.

Mermaid runs with `securityLevel: "strict"`. This reduces Mermaid-side risk, but the document is still rendered in a browser process.

Local file access is disabled by default. Passing `--allow-local-files` enables Chrome's `--allow-file-access-from-files` flag so local images and related assets can load.

Use these options only when:

- `--allow-html`: you wrote the Markdown or trust the source that produced it.
- `--allow-local-files`: the document needs local images or assets and the file paths are expected.

Use separate, locked-down infrastructure for strongly untrusted Markdown.
