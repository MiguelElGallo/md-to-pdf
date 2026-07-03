# Release checklist

Before publishing a release:

- Run `cargo fmt --check`.
- Run `cargo clippy --locked --all-targets -- -D warnings`.
- Run `cargo test --locked`.
- Run browser smoke tests for plain Markdown, valid Mermaid, and invalid Mermaid.
- Run `uv run --locked --group docs zensical build --clean --strict`.
- Run `actionlint .github/workflows/ci.yml .github/workflows/release.yml .github/workflows/docs.yml`.
- Verify the README quickstart from a fresh clone.
- Verify the CLI reference against `md-to-pdf --help`.
- Verify offline Mermaid rendering with `--mermaid-js`.
- Run the `Release` workflow with `tag=dry-run`.
- Confirm release archives and SHA-256 checksum files are produced.
