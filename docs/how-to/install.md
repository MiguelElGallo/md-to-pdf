---
icon: lucide/download
---

# Install md-to-pdf

Choose the **agent plugin** for a ready-to-use skill and MCP converter, or the **standalone CLI** for terminal scripts. Installing only the skill provides instructions, not the converter.

## Requirements

All conversion routes need Chrome, Chromium, or Microsoft Edge. The plugin additionally supports Python **3.11 or newer**, available as `python3` to the agent process, and a client with plugin support. Git is needed to fetch a repository marketplace. The standalone CLI does not require Python.

Before installing a plugin, run:

```sh
python3 --version
git --version
```

These checks must work from the environment that starts your agent, not only from a different terminal. On Windows, a working `python` or `py` command alone does not satisfy the plugin's `python3` launcher.

The first plugin conversion needs access to GitHub Releases to download a binary. Mermaid diagrams load a JavaScript library from a CDN by default; [offline use](#offline-use) requires a local bundle. Other remote document assets can also use the network.

## Install the agent plugin

### Codex CLI

```sh
codex plugin marketplace add MiguelElGallo/md-to-pdf
codex plugin add md-to-pdf@md-to-pdf
```

### GitHub Copilot CLI

```sh
copilot plugin marketplace add MiguelElGallo/md-to-pdf
copilot plugin install md-to-pdf@md-to-pdf
```

### VS Code

In a version supporting agent plugins, open the Command Palette, choose **Chat: Install Plugin From Source**, and enter `https://github.com/MiguelElGallo/md-to-pdf`. Python and the browser must be available to VS Code as well.

### Make your first PDF

Start a new session or chat after installation. Open a folder containing a Markdown file and ask:

> Convert report.md to a PDF using md-to-pdf and give me a link to the result.

The plugin includes the `convert-to-pdf` skill and the `convert_markdown_to_pdf` MCP tool. On first use it downloads its version-matched CLI binary, verifies the release SHA-256 checksum, and caches the executable in the plugin's persistent data directory (or the user cache when the client does not provide one). This download does not require administrator access or a Python package installation.

The default output is a sibling `report.pdf`. The skill checks whether that destination is already occupied before converting. Open the returned PDF and check text, diagrams, and page breaks.

To confirm installation independently of conversion:

```sh
codex plugin list
# Or, for Copilot:
copilot plugin list
```

An installed-plugin listing confirms discovery, not successful rendering. A completed conversion is the end-to-end check.

## Update the plugin

For Codex, refresh the marketplace and add the current version:

```sh
codex plugin marketplace upgrade md-to-pdf
codex plugin add md-to-pdf@md-to-pdf
```

For Copilot:

```sh
copilot plugin marketplace update md-to-pdf
copilot plugin update md-to-pdf
```

Start a new session, confirm the installed version, and convert a small document again. Codex command spelling is verified with `codex-cli 0.153.4`; it uses `marketplace upgrade`, not `plugin update`. For other client versions, check `codex plugin --help` and `codex plugin marketplace --help`. See the official [Copilot plugin reference](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-plugin-reference) for its commands.

In VS Code, use the client's plugin management UI to check for an update; if that version has no update action, reinstall from the same repository source and start a new chat.

By default, the MCP server uses its managed, version-matched binary, even if an older standalone CLI is on `PATH`. It does not upgrade that standalone CLI. If you set `MD_TO_PDF_BIN` or disable automatic installation, update your manually managed executable separately.

## Uninstall the plugin

Run the command for your client:

```sh
codex plugin remove md-to-pdf@md-to-pdf
# Or, for Copilot:
copilot plugin uninstall md-to-pdf
```

Optionally unregister this repository's marketplace after uninstalling the plugin:

```sh
codex plugin marketplace remove md-to-pdf
# Or, for Copilot:
copilot plugin marketplace remove md-to-pdf
```

In VS Code, remove the plugin through its plugin management UI. Start a new session afterward. Uninstalling does not delete your Markdown files or generated PDFs. Binary cache retention depends on the client; do not delete the client's entire cache or settings to remove this plugin.

## Skill-only setup

The portable skill is [`skills/convert-to-pdf/SKILL.md`](https://github.com/MiguelElGallo/md-to-pdf/blob/main/skills/convert-to-pdf/SKILL.md). Install that folder using your client's supported skill installation mechanism if you already manage your own tools.

A skill-only installation **does not** register the MCP server, download the `md-to-pdf` binary, or install a browser. With an installed CLI and shell access, the skill can use the CLI fallback. Without either the MCP tool or CLI, it can only explain the missing dependency. Use the full plugin above for automatic binary setup.

## Offline use

Prepare a compatible CLI binary from a verified release and a trusted Mermaid browser/UMD bundle that exposes `window.mermaid`. Set `MD_TO_PDF_BIN` to the executable's full path in the agent's environment before starting it. This avoids the first-conversion binary download.

Ask the agent to convert using the local bundle. The MCP arguments are:

```json
{
  "input": "/path/to/report.md",
  "output": "/path/to/report.pdf",
  "mermaid_js": "/path/to/vendor/mermaid.min.js",
  "virtual_time_budget_ms": 20000
}
```

Do not combine `mermaid_js` with `mermaid_url`. A local bundle avoids the Mermaid CDN; it does not make remote images, fonts, or stylesheets available offline, or remove the agent client's own connectivity requirements. See [Use Mermaid offline](use-local-mermaid.md) for the CLI equivalent.

## Troubleshooting

| Symptom | What to check |
| --- | --- |
| `plugin` command is unknown | Update your agent client to a version with plugin support. Check its `plugin --help` output. |
| Skill or tool is missing | Confirm installation with the client's plugin listing, then start a new session. Installing only a skill does not register MCP. |
| MCP fails to launch / `python3` not found | Run `python3 --version` from the agent's launch environment. On Windows, configure your Python installation to expose a real `python3` command, or set the client's MCP launcher override to the installed Python executable if supported. A shell alias in a different terminal is not enough. Restart the client. |
| Browser not found | Install Chrome, Chromium, or Edge, or pass its executable path as the tool's `browser` option. `MD_TO_PDF_BROWSER` is the environment equivalent; see [Choose a browser](choose-a-browser.md). |
| Download fails or platform is unsupported | Check GitHub Releases access. Supply a compatible, verified executable through `MD_TO_PDF_BIN` when automatic installation is unavailable. |
| Checksum mismatch | Stop and retry the download from the official release. Never disable verification to run the mismatched file. |
| Old CLI after an update | Restart the agent and check `MD_TO_PDF_BIN` and `MD_TO_PDF_AUTO_INSTALL`. A manually selected executable is not upgraded by the plugin. |
| Mermaid diagrams are missing | Check syntax and CDN access, or use `mermaid_js`. For slow diagrams, increase `virtual_time_budget_ms` within 1–60000. Set `keep_html: true` when you need the generated HTML for diagnosis. |

`keep_html` writes a sibling HTML file that can contain sensitive document content. Raw HTML and local-file browser access are disabled by default; do not enable `allow_html` or `allow_local_files` as a generic workaround. See the [safety model](../explanation/safety-model.md).

To disable automatic downloads, set `MD_TO_PDF_AUTO_INSTALL=0` before launching the agent. The server can then use `md-to-pdf` on `PATH`, without guaranteeing its version matches the plugin. An explicit `MD_TO_PDF_BIN` takes precedence in either mode. Use the client's supported environment configuration; do not edit cached plugin files, because an update can replace them.

## Install the standalone CLI

Download the archive for your platform and its matching `.sha256` file from the [latest release](https://github.com/MiguelElGallo/md-to-pdf/releases/latest). The names below use `<tag>` for a release tag, such as `v0.5.0`.

| Platform | Archive |
| --- | --- |
| macOS Apple Silicon | `md-to-pdf-<tag>-aarch64-apple-darwin.zip` |
| macOS Intel | `md-to-pdf-<tag>-x86_64-apple-darwin.zip` |
| Linux x86_64 | `md-to-pdf-<tag>-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `md-to-pdf-<tag>-x86_64-pc-windows-msvc.zip` |

### macOS installer

```sh
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | sh
```

The script detects Apple Silicon or Intel, verifies the matching checksum, and installs to `/usr/local/bin` using administrator access. To pin a release:

```sh
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | MD_TO_PDF_VERSION=v0.5.0 sh
```

For a no-admin installation, verify and extract the matching release archive, then run the extracted binary directly or place it in a user-owned directory on `PATH`. Check the release notes for macOS signing and notarization status.

### Linux: verify and install without admin access

In the folder containing the downloaded archive and checksum:

```sh
VERSION="v0.5.0" # use the tag you downloaded
sha256sum -c "md-to-pdf-${VERSION}-x86_64-unknown-linux-gnu.sha256" && \
tar -xzf "md-to-pdf-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" && \
mkdir -p "$HOME/.local/bin" && \
install "md-to-pdf-${VERSION}-x86_64-unknown-linux-gnu/md-to-pdf" "$HOME/.local/bin/md-to-pdf" && \
"$HOME/.local/bin/md-to-pdf" --version
```

Add `$HOME/.local/bin` to your shell's `PATH` if it is not already present. The chained commands stop before extraction or installation if checksum verification fails.

### Windows: verify and run

In PowerShell, from the download folder:

```powershell
$version = "v0.5.0" # use the tag you downloaded
$archive = ".\md-to-pdf-$version-x86_64-pc-windows-msvc.zip"
$checksum = ".\md-to-pdf-$version-x86_64-pc-windows-msvc.sha256"
$expected = ((Get-Content $checksum -Raw).Trim() -split '\s+')[0]
$actual = (Get-FileHash $archive -Algorithm SHA256).Hash
if ($actual -ne $expected) { throw "Checksum mismatch; do not run this archive." }
Expand-Archive $archive -DestinationPath .
.\md-to-pdf-$version-x86_64-pc-windows-msvc\md-to-pdf.exe --version
```

Run the executable by its full path, or put its directory on your user `PATH`. Administrator access is not required to run it from a user-owned folder.

### Install from source

With Rust and Cargo installed:

```sh
cargo install --git https://github.com/MiguelElGallo/md-to-pdf --tag v0.5.0
md-to-pdf --version
```

From a local checkout, use `cargo install --path .`. For development without installing, use `cargo run -- fixtures/basic.md --output /tmp/basic.pdf`.

### Verify, update, or remove the CLI

After installation, convert a file with `md-to-pdf report.md` and open the generated PDF. Update a manually installed binary by verifying and installing the new release archive; Cargo installs can use the source command above with the new tag and `--force`.

For a Cargo install, use `cargo uninstall md-to-pdf`. For an archive install, remove only the executable you installed after confirming its location. Neither operation should remove your source documents or PDFs.
