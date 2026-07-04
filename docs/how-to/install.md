# Install md-to-pdf

This guide shows you how to install the CLI from a GitHub Release or from this repository.

## Install from a release archive

Download the archive for your platform from the [latest release](https://github.com/MiguelElGallo/md-to-pdf/releases/latest):

Choose the archive that matches your operating system and CPU.

| Platform | Archive |
| --- | --- |
| macOS Apple Silicon | `md-to-pdf-v0.1.1-aarch64-apple-darwin.zip` |
| macOS Intel | `md-to-pdf-v0.1.1-x86_64-apple-darwin.zip` |
| Linux x86_64 | `md-to-pdf-v0.1.1-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `md-to-pdf-v0.1.1-x86_64-pc-windows-msvc.zip` |

Download the matching `.sha256` file too.

## Install on macOS

Run:

```sh
curl -fsSL https://raw.githubusercontent.com/MiguelElGallo/md-to-pdf/main/scripts/install-macos.sh | sh
```

The installer detects Apple Silicon vs Intel, downloads the latest macOS archive and matching checksum, verifies the checksum, and installs `md-to-pdf` to `/usr/local/bin`.

Check the release notes for the current macOS signing and notarization status.

## Verify and install on Linux

Run:

```sh
shasum -a 256 -c md-to-pdf-v0.1.1-x86_64-unknown-linux-gnu.sha256
tar -xzf md-to-pdf-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
sudo install md-to-pdf-v0.1.1-x86_64-unknown-linux-gnu/md-to-pdf /usr/local/bin/md-to-pdf
md-to-pdf --help
```

## Verify and run on Windows

Run in PowerShell:

```powershell
Get-FileHash .\md-to-pdf-v0.1.1-x86_64-pc-windows-msvc.zip -Algorithm SHA256
Expand-Archive .\md-to-pdf-v0.1.1-x86_64-pc-windows-msvc.zip
.\md-to-pdf-v0.1.1-x86_64-pc-windows-msvc\md-to-pdf.exe --help
```

Open the matching `.sha256` file and confirm its hash matches the `Get-FileHash` output before running the executable.

## Install from source

Run:

```sh
cargo install --path .
```

Confirm the command is available:

```sh
md-to-pdf --help
```

## Build without installing

For development, run:

```sh
cargo build
```

Then run the binary through Cargo:

```sh
cargo run -- fixtures/basic.md --output /tmp/basic.pdf
```
