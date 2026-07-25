use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use std::process::Command;

/// Options for HTML2PDF rendering
#[derive(Debug, Clone)]
pub struct Html2PdfOptions {
    pub page_size: String,
    pub allow_local_files: bool,
}

/// Render HTML to PDF using the HTML2PDF library
///
/// HTML2PDF is a Python-based HTML to PDF converter. It must be installed
/// separately and available in the system PATH.
///
/// Installation:
/// ```sh
/// pip install html2pdf
/// ```
///
/// # Arguments
/// * `html_path` - Path to the HTML file to convert
/// * `pdf_path` - Path where the PDF should be written
/// * `options` - HTML2PDF rendering options
pub fn render_to_pdf(
    html_path: &Utf8Path,
    pdf_path: &Utf8Path,
    options: &Html2PdfOptions,
) -> Result<()> {
    // Verify HTML2PDF is installed
    if !is_html2pdf_installed()? {
        return Err(anyhow!(
            "HTML2PDF is not installed. Install it with: pip install html2pdf"
        ));
    }

    if let Some(parent) = pdf_path.parent() {
        if !parent.as_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create output directory {parent}"))?;
        }
    }

    // Get absolute paths
    let html_path = html_path
        .canonicalize_utf8()
        .with_context(|| format!("failed to resolve {html_path}"))?;
    let pdf_path = pdf_path
        .canonicalize_utf8()
        .with_context(|| format!("failed to resolve {pdf_path}"))?;

    // Build HTML2PDF command
    let mut command = Command::new("html2pdf");
    command.arg("--input-file").arg(html_path.as_str());
    command.arg("--output-file").arg(pdf_path.as_str());

    // Map page size
    match options.page_size.to_uppercase().as_str() {
        "A4" => command.arg("--page-size").arg("A4"),
        "LETTER" => command.arg("--page-size").arg("Letter"),
        "LEGAL" => command.arg("--page-size").arg("Legal"),
        "A3" => command.arg("--page-size").arg("A3"),
        "A5" => command.arg("--page-size").arg("A5"),
        other => {
            // Warn but continue with the provided size
            eprintln!("Warning: HTML2PDF may not support page size '{other}', attempting anyway", );
            command.arg("--page-size").arg(other)
        }
    };

    // Enable local file access if requested
    if options.allow_local_files {
        command.arg("--enable-local");
    }

    let output = command
        .output()
        .context("failed to execute html2pdf command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "HTML2PDF conversion failed: {}",
            stderr.trim_end()
        ));
    }

    // Verify the PDF was created and is not empty
    if !pdf_path.exists() {
        return Err(anyhow!("HTML2PDF did not create output file: {pdf_path}"));
    }

    let file_size = fs::metadata(&pdf_path)
        .with_context(|| format!("failed to check PDF file size {pdf_path}"))?
        .len();

    if file_size == 0 {
        return Err(anyhow!("HTML2PDF created an empty PDF: {pdf_path}"));
    }

    Ok(())
}

/// Check if HTML2PDF is available in the system PATH
fn is_html2pdf_installed() -> Result<bool> {
    match Command::new("html2pdf").arg("--version").output() {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html2pdf_options_debug_format() {
        let options = Html2PdfOptions {
            page_size: "A4".to_string(),
            allow_local_files: false,
        };
        assert_eq!(
            format!("{:?}", options),
            r#"Html2PdfOptions { page_size: "A4", allow_local_files: false }"#
        );
    }
}
