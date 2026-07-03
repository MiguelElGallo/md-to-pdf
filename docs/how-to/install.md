# Install md-to-pdf

This guide shows you how to install the CLI from a GitHub Release or from this repository.

## Install from a release archive

Download the archive for your platform from the [latest release](https://github.com/MiguelElGallo/md-to-pdf/releases/latest):

| Platform | Archive |
| --- | --- |
| macOS Apple Silicon | `md-to-pdf-v0.1.1-aarch64-apple-darwin.zip` |
| macOS Intel | `md-to-pdf-v0.1.1-x86_64-apple-darwin.zip` |
| Linux x86_64 | `md-to-pdf-v0.1.1-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `md-to-pdf-v0.1.1-x86_64-pc-windows-msvc.zip` |

Download the matching `.sha256` file too.

## Verify and install on macOS or Linux

Run:

```sh
shasum -a 256 -c md-to-pdf-v0.1.1-aarch64-apple-darwin.sha256
unzip md-to-pdf-v0.1.1-aarch64-apple-darwin.zip
sudo install md-to-pdf-v0.1.1-aarch64-apple-darwin/md-to-pdf /usr/local/bin/md-to-pdf
md-to-pdf --help
```

Choose the archive name that matches your platform. Linux uses `tar -xzf` instead of `unzip`.

## Verify and run on Windows

Run in PowerShell:

```powershell
Get-FileHash .\md-to-pdf-v0.1.1-x86_64-pc-windows-msvc.zip -Algorithm SHA256
Expand-Archive .\md-to-pdf-v0.1.1-x86_64-pc-windows-msvc.zip
.\md-to-pdf-v0.1.1-x86_64-pc-windows-msvc\md-to-pdf.exe --help
```

Compare the printed hash with the matching `.sha256` file.

## macOS Gatekeeper note

macOS release artifacts are `.zip` archives. When Apple Developer ID secrets are configured for the release workflow, macOS binaries are signed with the hardened runtime and the zip archives are accepted by Apple's notary service. Unsigned macOS publishing is blocked unless a maintainer explicitly allows it in a manual release dispatch. If a release note says the macOS artifacts are unsigned, expect Gatekeeper to treat the archive as unsigned.

Only remove quarantine attributes for a binary after verifying the checksum and deciding that you trust the source:

```sh
xattr -dr com.apple.quarantine /usr/local/bin/md-to-pdf
```

For a signed release, the release notes should say the macOS artifacts were accepted by Apple's notary service. If Gatekeeper still warns on a signed release, check that you downloaded the matching archive and checksum from the same release.

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
