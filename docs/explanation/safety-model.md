---
icon: lucide/shield-check
---

# Safety model

What risks do the defaults reduce, and what remains outside scope?

The CLI has conservative defaults, but it is not a complete sandbox for hostile documents.

Raw HTML in Markdown is escaped by default. Passing `--allow-html` changes that behavior and should be reserved for trusted Markdown.

Mermaid runs with `securityLevel: "strict"`. This reduces Mermaid-side risk, but the document is still rendered in a browser process.

Local file access is disabled by default. Passing `--allow-local-files` enables Chrome's `--allow-file-access-from-files` flag so local images and related assets can load.

Remote assets referenced by Markdown, CSS, or raw HTML are blocked by a Content Security Policy by default. The fixed jsDelivr URL used by the built-in Mermaid renderer is the only remote script exception. Passing `--allow-remote-assets` restores HTTP(S) document resources, including access to services on private, link-local, and loopback networks. Treat it as a network-trust decision, not a rendering workaround. A custom `--mermaid-url` requires this opt-in; use `--mermaid-js` for secure offline rendering.

Use these options only when:

- `--allow-html`: you wrote the Markdown or trust the source that produced it.
- `--allow-local-files`: the document needs local images or assets and the file paths are expected.
- `--allow-remote-assets`: the document and its stylesheet are trusted to choose every network destination the browser may contact.

Use separate, locked-down infrastructure for strongly untrusted Markdown.
