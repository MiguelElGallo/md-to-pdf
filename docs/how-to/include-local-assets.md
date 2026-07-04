---
icon: lucide/image
---

# Include local assets

This guide shows you how to render Markdown that references local images or other local files.

## Reference the asset from Markdown

Use a relative path from the Markdown file:

```markdown
![Architecture](images/architecture.png)
```

## Allow local file access

Run:

```sh
md-to-pdf page-with-image.md --allow-local-files
```

!!! warning
	Use `--allow-local-files` only for Markdown files you trust. It allows the rendering browser to load local files referenced by the document.

This option passes Chrome's `--allow-file-access-from-files` flag to the rendering browser.

If the asset is missing, confirm the path is relative to the Markdown file, then rerun with `--keep-html` and inspect the generated HTML. See [Debug rendering](debug-rendering.md).
