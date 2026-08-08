use anyhow::{anyhow, Context, Result};
use lopdf::{Document, Object};
use std::path::Path;

const POINTS_PER_INCH: f64 = 72.0;
const MM_PER_INCH: f64 = 25.4;

/// Load a PDF document from disk.
pub fn load_pdf(path: &Path) -> Result<Document> {
    Document::load(path).with_context(|| format!("failed to load PDF {path:?}"))
}

/// Return the number of pages in the document.
pub fn page_count(doc: &Document) -> usize {
    doc.get_pages().len()
}

/// Extract all text from the document in page order.
///
/// `lopdf::Document::extract_text` expects 1-indexed page numbers, which match
/// the keys returned by `doc.get_pages()`.
pub fn extract_text(doc: &Document) -> Result<String> {
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    doc.extract_text(&pages)
        .with_context(|| "failed to extract text from PDF")
}

/// Return true if the extracted text contains `needle`.
pub fn contains_text(doc: &Document, needle: &str) -> Result<bool> {
    Ok(extract_text(doc)?.contains(needle))
}

/// Return the width and height of the given 1-indexed page in millimeters.
pub fn page_size_mm(doc: &Document, page_number: usize) -> Result<(f64, f64)> {
    let pages = doc.get_pages();
    let object_id = pages
        .get(&(page_number as u32))
        .ok_or_else(|| anyhow!("page {page_number} does not exist in PDF"))?;
    let page_dict = doc
        .get_object(*object_id)
        .context("failed to read page object")?
        .as_dict()
        .context("page object is not a dictionary")?;
    let media_box = page_dict
        .get(b"MediaBox")
        .context("page is missing MediaBox")?;

    let bounds = media_box.as_array().context("MediaBox is not an array")?;
    if bounds.len() != 4 {
        anyhow::bail!("MediaBox does not contain four values");
    }

    let lower_left_x = as_f64(&bounds[0])?;
    let lower_left_y = as_f64(&bounds[1])?;
    let upper_right_x = as_f64(&bounds[2])?;
    let upper_right_y = as_f64(&bounds[3])?;

    let width_pt = upper_right_x - lower_left_x;
    let height_pt = upper_right_y - lower_left_y;

    Ok((points_to_mm(width_pt), points_to_mm(height_pt)))
}

fn as_f64(object: &Object) -> Result<f64> {
    match object {
        Object::Integer(value) => Ok(*value as f64),
        Object::Real(value) => Ok(f64::from(*value)),
        _ => Err(anyhow!("expected numeric PDF object, got {:?}", object)),
    }
}

fn points_to_mm(points: f64) -> f64 {
    points * MM_PER_INCH / POINTS_PER_INCH
}
