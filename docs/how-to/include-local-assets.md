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

Use this option only for trusted local documents. It passes Chrome's `--allow-file-access-from-files` flag to the rendering browser.
