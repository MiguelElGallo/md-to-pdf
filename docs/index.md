# md-to-pdf documentation

`md-to-pdf` converts one Markdown file into one PDF. Mermaid diagrams in fenced code blocks are rendered by a Chromium-family browser before the PDF is written.

## Start here

- New to the tool: follow [Create your first PDF](tutorials/first-pdf.md).
- Need a specific result: use the [how-to guides](how-to/install.md).
- Need exact flags and defaults: consult [CLI options](reference/cli.md).
- Want to understand the design: read [Rendering pipeline](explanation/rendering-pipeline.md).

## Requirements

- Rust with Cargo.
- Chrome, Chromium, or Microsoft Edge.
- Internet access for Mermaid diagrams by default.

Plain Markdown conversion does not need network access. Mermaid diagrams use jsDelivr by default unless you provide a local Mermaid browser bundle.

## Documentation map

This documentation follows Diataxis:

- Tutorials are lessons. They guide you through a successful first experience.
- How-to guides solve practical tasks.
- Reference describes commands, options, defaults, and errors.
- Explanation discusses the reasons behind the design.
