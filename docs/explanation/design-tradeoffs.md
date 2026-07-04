# Design tradeoffs

Why choose this dependency shape for the first version?

The project deliberately avoids a pure Rust PDF renderer for the first version.

Rust-native PDF crates are useful for constructing PDFs, but they do not provide modern browser layout or Mermaid rendering. `wkhtmltopdf` has weaker modern JavaScript support. Pandoc is powerful, but it adds a large external dependency and still needs Mermaid integration.

The current design keeps the Rust part small and testable while delegating JavaScript, CSS, SVG, and PDF layout to a Chromium-family browser.

The main cost is the browser dependency. Users need Chrome, Chromium, or Edge installed, and CI jobs need to make that dependency explicit.
