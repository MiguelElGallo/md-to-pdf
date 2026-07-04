# md-to-pdf documentation

Convert one Markdown file into one PDF, with Mermaid diagrams rendered before the PDF is written.

`md-to-pdf` turns Markdown into browser-ready HTML, waits for Mermaid diagrams to finish, and asks Chrome, Chromium, or Edge to print the page as a PDF.

```text
Markdown file -> HTML document -> browser rendering -> PDF file
```

## Find your way around

This documentation follows the [Diataxis](https://diataxis.fr/) framework. Pick the section that matches what you need right now.

### Tutorials

Learning-oriented lessons. Start here if you want a guided first success.

[Create your first PDF](tutorials/first-pdf.md)

### How-to guides

Task-oriented recipes for installation, browser selection, custom CSS, local assets, Mermaid, and CI.

[How-to guides](how-to/index.md)

### Reference

Lookup material for commands, options, defaults, supported Markdown, configuration, and errors.

[CLI options](reference/cli.md)

### Explanation

Background on why the tool uses a browser, how Mermaid rendering works, and what the safety defaults protect.

[Rendering pipeline](explanation/rendering-pipeline.md)

## Requirements

You need Chrome, Chromium, or Microsoft Edge. Plain Markdown conversion works offline; Mermaid diagrams use jsDelivr by default unless you provide a local Mermaid browser bundle.

See [Install md-to-pdf](how-to/install.md) for platform-specific installation steps.

## Why use a browser?

Markdown-to-PDF tools often struggle when diagrams need JavaScript and browser layout. `md-to-pdf` keeps the pipeline HTML-first so Mermaid and print CSS run in the same environment that creates the PDF.

For the design details, read [Rendering pipeline](explanation/rendering-pipeline.md) and [Design tradeoffs](explanation/design-tradeoffs.md).

## Maintainers

Maintainer pages cover release operations such as macOS signing and the release checklist. They are contributor-facing, separate from the user-facing Diataxis sections.
