# Create your first PDF

In this tutorial, we will create a small Markdown file and convert it to a PDF.

## Check the command

From the repository root, run:

```sh
cargo run -- --help
```

You should see help output that includes `--output`, `--browser`, and `--page-size`.

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
cargo run -- example.md
```

You should see:

```text
Wrote example.pdf
```

Now check that `example.pdf` exists next to `example.md`.

## Repeat it

Run the same command again:

```sh
cargo run -- example.md
```

The command rewrites `example.pdf`. This repetition is useful: it confirms that converting a Markdown file is a normal, repeatable workflow.
