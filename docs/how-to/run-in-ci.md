# Run md-to-pdf in CI

This guide shows you how to run `md-to-pdf` in automation.

## Install Rust and find Chrome

In GitHub Actions, use a Chromium-family browser path from the runner:

```yaml
- uses: dtolnay/rust-toolchain@stable
- name: Find Chrome
  id: chrome
  shell: bash
  run: |
    browser="$(command -v google-chrome || command -v chromium || command -v chromium-browser)"
    if [[ -z "$browser" ]]; then
      echo "No Chromium-family browser found"
      exit 1
    fi
    echo "path=$browser" >> "$GITHUB_OUTPUT"
- name: Convert docs
  env:
    MD_TO_PDF_BROWSER: ${{ steps.chrome.outputs.path }}
  run: cargo run --release -- fixtures/basic.md --output guide.pdf
```

This example tests the CLI from the repository. If your workflow downloads a release binary instead, run `md-to-pdf fixtures/basic.md --output guide.pdf`.

## Prefer local Mermaid in CI

For reproducible CI, pass `--mermaid-js` with a pinned local bundle instead of using the default CDN URL.
