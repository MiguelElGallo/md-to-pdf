pub mod browser;
pub mod document;
pub mod markdown;

use anyhow::{bail, Result};
use camino::{Utf8Path, Utf8PathBuf};

pub fn default_output_path(input: &Utf8Path) -> Result<Utf8PathBuf> {
    if input.file_name().is_none() {
        bail!("input path must point to a Markdown file");
    }

    Ok(input.with_extension("pdf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_output_path_to_pdf_extension() {
        let output = default_output_path(Utf8Path::new("docs/guide.md")).unwrap();

        assert_eq!(output, Utf8PathBuf::from("docs/guide.pdf"));
    }
}
