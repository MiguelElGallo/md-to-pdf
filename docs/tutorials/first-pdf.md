---
icon: lucide/file-plus-2
---

# Create your first PDF

In this tutorial, we will create a small Markdown file and convert it to a PDF.

By the end, you will have an `example.pdf` file next to the Markdown source.

## Make sure the command is available

Run:

```sh
md-to-pdf --help
```

!!! tip
	If you are running from the repository instead of an installed release, replace `md-to-pdf` with `cargo run --`.

From the repository, check the command with:

```sh
cargo run -- --help
```

You should see the command help.

## Create a Markdown file

Create `example.md`:

```markdown
# First PDF

This PDF was created from Markdown.

- Headings become PDF headings.
- Lists stay as lists.
- Inline code like `md-to-pdf` stays readable.
```

## Convert it

Run:

```sh
md-to-pdf example.md
```

From the repository, use `cargo run -- example.md` instead.

You should see:

```text
Wrote example.pdf
```

Now check that `example.pdf` exists next to `example.md`.

## Repeat it

Run the same command again:

```sh
md-to-pdf example.md
```

The command rewrites `example.pdf`. This repetition is useful: it confirms that converting a Markdown file is a normal, repeatable workflow.

## Recap

You created `example.md`, converted it with `md-to-pdf example.md`, and confirmed that `example.pdf` was written next to the source file.

## Next steps

- Add a diagram with [Render Mermaid diagrams](markdown-with-mermaid.md).
- Look up output, browser, and CSS flags in [CLI options](../reference/cli.md).
