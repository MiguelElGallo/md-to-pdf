use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use md_to_pdf::browser::{file_url, print_to_pdf, BrowserOptions};
use md_to_pdf::default_output_path;
use md_to_pdf::document::{render_document, DocumentOptions, MermaidSource, DEFAULT_MERMAID_URL};
use md_to_pdf::markdown::{markdown_to_body, HtmlOptions};
use std::fs;
use tempfile::tempdir;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Markdown file to convert.
    input: Utf8PathBuf,

    /// PDF output path. Defaults to the input file name with a .pdf extension.
    #[arg(short, long)]
    output: Option<Utf8PathBuf>,

    /// Chrome, Chromium, or Edge executable to use.
    #[arg(long, env = "MD_TO_PDF_BROWSER")]
    browser: Option<Utf8PathBuf>,

    /// CSS page size, for example A4, Letter, or Legal.
    #[arg(long, default_value = "A4")]
    page_size: String,

    /// Extra CSS file to append after the built-in print styles.
    #[arg(long)]
    css: Option<Utf8PathBuf>,

    /// Mermaid ES module URL. Use --mermaid-js for offline rendering.
    #[arg(long, default_value = DEFAULT_MERMAID_URL, conflicts_with = "mermaid_js")]
    mermaid_url: String,

    /// Local Mermaid browser bundle to embed. Use the UMD/browser build that exposes window.mermaid.
    #[arg(long, conflicts_with = "mermaid_url")]
    mermaid_js: Option<Utf8PathBuf>,

    /// Let raw HTML in Markdown pass through to the generated document.
    #[arg(long)]
    allow_html: bool,

    /// Pass Chrome's --allow-file-access-from-files flag for local assets referenced by Markdown.
    #[arg(long)]
    allow_local_files: bool,

    /// Browser virtual time budget in milliseconds for Mermaid and layout before PDF printing.
    #[arg(long, default_value_t = 10_000)]
    virtual_time_budget: u64,

    /// Write the generated HTML next to the PDF for debugging.
    #[arg(long)]
    keep_html: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> Result<()> {
    validate_input(&cli.input)?;

    let output = cli
        .output
        .clone()
        .map(Ok)
        .unwrap_or_else(|| default_output_path(&cli.input))?;
    let markdown =
        fs::read_to_string(&cli.input).with_context(|| format!("failed to read {}", cli.input))?;
    let body = markdown_to_body(
        &markdown,
        &HtmlOptions {
            allow_html: cli.allow_html,
        },
    );
    let custom_css = match &cli.css {
        Some(path) => Some(
            fs::read_to_string(path).with_context(|| format!("failed to read CSS file {path}"))?,
        ),
        None => None,
    };
    let mermaid_source = match &cli.mermaid_js {
        Some(path) => MermaidSource::InlineScript(
            fs::read_to_string(path)
                .with_context(|| format!("failed to read Mermaid script {path}"))?,
        ),
        None => MermaidSource::EsModuleUrl(cli.mermaid_url.clone()),
    };
    let base_href = input_base_href(&cli.input)?;
    let document = render_document(
        &body,
        &DocumentOptions {
            title: cli.input.file_stem().unwrap_or("Document").to_string(),
            base_href,
            page_size: cli.page_size.clone(),
            custom_css,
            mermaid_source,
        },
    );

    let temp_dir = tempdir().context("failed to create temporary directory")?;
    let html_path = if cli.keep_html {
        output.with_extension("html")
    } else {
        Utf8PathBuf::from_path_buf(temp_dir.path().join("document.html")).map_err(|path| {
            anyhow::anyhow!("temporary path is not valid UTF-8: {}", path.display())
        })?
    };
    if let Some(parent) = html_path.parent() {
        if !parent.as_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create HTML output directory {parent}"))?;
        }
    }
    fs::write(&html_path, document).with_context(|| format!("failed to write {html_path}"))?;

    print_to_pdf(
        &html_path,
        &output,
        &BrowserOptions {
            browser: cli.browser.clone(),
            virtual_time_budget_ms: cli.virtual_time_budget,
            allow_local_files: cli.allow_local_files,
        },
    )?;

    println!("Wrote {output}");
    Ok(())
}

fn validate_input(input: &Utf8Path) -> Result<()> {
    if !input.exists() {
        anyhow::bail!("input file does not exist: {input}");
    }
    if input.is_dir() {
        anyhow::bail!("input path is a directory: {input}");
    }
    Ok(())
}

fn input_base_href(input: &Utf8Path) -> Result<Option<String>> {
    let parent = input.parent().unwrap_or_else(|| Utf8Path::new("."));
    let url = file_url(parent)?;
    Ok(Some(url.to_string()))
}
