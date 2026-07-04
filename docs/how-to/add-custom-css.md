---
icon: lucide/paintbrush
---

# Add print CSS

This guide shows you how to add custom CSS to the generated PDF.

## Create a stylesheet

Create `print.css`:

```css
body {
  font-size: 11pt;
}

h1 {
  color: #0f766e;
}
```

## Convert with the stylesheet

Run:

```sh
md-to-pdf fixtures/basic.md --css print.css
```

The custom CSS is appended after the built-in print styles.

You should see:

```text
Wrote fixtures/basic.pdf
```

Open the PDF and check that the heading uses the custom color.

!!! tip
  Use print-focused CSS here. Browser-only interactive styles will not affect the generated PDF.

For layout problems, keep the generated HTML with [Debug rendering](debug-rendering.md).
