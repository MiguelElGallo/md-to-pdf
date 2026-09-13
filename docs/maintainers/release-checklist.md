---
icon: lucide/list-checks
---

# Release checklist

Before publishing a release:

- Run `cargo fmt --check`.
- Run `cargo clippy --locked --all-targets -- -D warnings`.
- Run `cargo test --locked`.
- Run `python3 -m unittest discover -s mcp_server -p 'test_*.py'`.
- Run `ruff check mcp_server plugins/md-to-pdf/mcp_server` and `ty check mcp_server plugins/md-to-pdf/mcp_server`.
- Confirm the packaged server and skill match their canonical copies (covered by packaging tests).
- Build the CLI, then run `MD_TO_PDF_RUN_INSTALL_JOURNEY=1 MD_TO_PDF_JOURNEY_BINARY=target/debug/md-to-pdf python3 -m unittest mcp_server.test_install_journey -v` (use `.exe` on Windows and your shell's environment-variable syntax). This exercises isolated installation, three actual conversions, and a simulated patch upgrade from local release-format fixtures.
- Confirm the install journey and the shipped `python3` manifest-launcher stdio conversion pass on Linux, macOS, and Windows in CI. Simulated upgrade tests do not replace public release read-back.
- Run browser smoke tests for plain Markdown, valid Mermaid, and invalid Mermaid.
- Run `uv run --locked --group docs zensical build --clean --strict`.
- Run `actionlint .github/workflows/ci.yml .github/workflows/release.yml .github/workflows/docs.yml`.
- Verify the README quickstart from a fresh clone.
- Verify the CLI reference against `md-to-pdf --help`.
- Verify offline Mermaid rendering with `--mermaid-js`.
- Verify the MCP `mermaid_js`, `keep_html`, and `virtual_time_budget_ms` options with a trusted local Mermaid browser bundle.
- Complete an independent peer review and security review of the release diff; resolve release-blocking findings before tagging.
- Run the `Release` workflow with `tag=dry-run`.
- Confirm release archives and SHA-256 checksum files are produced.
- Confirm release artifact attestations are produced.
- After publication, install from clean Codex/Copilot marketplace configurations and execute the cached MCP server to generate a PDF with the public release binary. Confirm the installed skill, server, and binary versions.
- Download each archive from the dry run or release artifacts, verify its checksum, extract it, and run `md-to-pdf --version` and `md-to-pdf --help`.
- Confirm release notes state the actual macOS signing/notarization status for this release.
- Confirm macOS release artifacts are `.zip` archives.
- If Apple signing secrets are configured, verify `codesign` and `notarytool --wait` pass in the release workflow before publishing. Do not use `spctl --type execute` as a gate for the standalone CLI; it expects an app bundle.
- If Apple signing secrets are not configured, a tag push must fail before publishing. Use manual dispatch with `allow_unsigned_macos=true` only for an intentionally unsigned release.
- If an unsigned macOS release is intentionally allowed, verify the release notes clearly say macOS artifacts are unsigned and not notarized.
