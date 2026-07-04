---
icon: lucide/lightbulb
---

# Explanation

Explanation pages describe why `md-to-pdf` works the way it does.

Read these when you want the design background behind browser rendering, Mermaid handling, safety defaults, and dependency choices.

## Topics

<div class="grid cards" markdown>

-   :lucide-workflow: **[Rendering pipeline](rendering-pipeline.md)**

	---

	Why conversion goes through HTML and a Chromium-family browser.

-   :lucide-chart-network: **[Mermaid rendering](mermaid-rendering.md)**

	---

	Why Mermaid is rendered at browser time and how readiness is detected.

-   :lucide-shield-check: **[Safety model](safety-model.md)**

	---

	What the defaults reduce, and what remains outside scope.

-   :lucide-scale: **[Design tradeoffs](design-tradeoffs.md)**

	---

	Why this first version depends on a browser instead of pure Rust PDF rendering.

</div>