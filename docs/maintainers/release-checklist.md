---
icon: lucide/list-checks
---

# Release checklist

Before publishing a release:

- Run `cargo fmt --check`.
- Run `cargo clippy --locked --all-targets -- -D warnings`.
- Run `cargo test --locked`.
- Run browser smoke tests for plain Markdown, valid Mermaid, invalid Mermaid, page size, and raw-HTML escaping.
- Verify browser smoke tests assert on PDF page count, extracted text, and page size (not just file size).
- Run `uv run --locked --group docs zensical build --clean --strict`.
- Run `actionlint .github/workflows/ci.yml .github/workflows/release.yml .github/workflows/docs.yml`.
- Verify the README quickstart from a fresh clone.
- Verify the CLI reference against `md-to-pdf --help`.
- Verify offline Mermaid rendering with `--mermaid-js`.
- If `DEFAULT_MERMAID_URL` in `src/document.rs` was changed, update the matching version in `.github/workflows/ci.yml`.
- Run the `Release` workflow with `tag=dry-run`.
- Confirm release archives and SHA-256 checksum files are produced.
- Confirm release artifact attestations are produced.
- Download each archive from the dry run or release artifacts, verify its checksum, extract it, and run `md-to-pdf --version` and `md-to-pdf --help`.
- Confirm release notes state the actual macOS signing/notarization status for this release.
- Confirm macOS release artifacts are `.zip` archives.
- If Apple signing secrets are configured, verify `codesign`, `notarytool`, and `spctl` pass in the release workflow before publishing.
- If Apple signing secrets are not configured, a tag push must fail before publishing. Use manual dispatch with `allow_unsigned_macos=true` only for an intentionally unsigned release.
- If an unsigned macOS release is intentionally allowed, verify the release notes clearly say macOS artifacts are unsigned and not notarized.
