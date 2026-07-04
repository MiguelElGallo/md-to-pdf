---
icon: lucide/file-code
---

# Markdown support

Markdown is parsed with `pulldown-cmark`.

## Enabled Markdown features

- Headings.
- Paragraphs.
- Lists.
- Links.
- Images.
- Tables.
- Footnotes.
- Strikethrough.
- Task lists.
- Heading attributes.
- Fenced code blocks.

## Mermaid fences

Fenced code blocks whose first info-string word is `mermaid` are rendered as Mermaid diagrams:

````markdown
```mermaid
graph TD
  A --> B
```
````

Non-Mermaid code fences remain code blocks.

## Raw HTML

Raw HTML is escaped by default. Pass `--allow-html` only for trusted Markdown.
