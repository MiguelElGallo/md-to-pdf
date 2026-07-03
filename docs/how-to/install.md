# Install md-to-pdf

This guide shows you how to install the CLI from this repository.

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
