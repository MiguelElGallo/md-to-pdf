use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

mod pdf;

#[test]
fn help_includes_core_options() {
    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--output"))
        .stdout(predicate::str::contains("--mermaid-js"))
        .stdout(predicate::str::contains("--browser"));
}

#[test]
fn missing_input_fails_before_browser_discovery() {
    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .arg("missing.md")
        .assert()
        .failure()
        .stderr(predicate::str::contains("input file does not exist"));
}

#[test]
fn missing_local_mermaid_bundle_fails_before_browser_discovery() {
    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/mermaid-flowchart.md",
            "--mermaid-js",
            "missing-mermaid.js",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read Mermaid script"));
}

#[test]
fn invalid_browser_path_fails_clearly() {
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("out.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--output",
            output.to_str().unwrap(),
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to start browser"));
}

#[test]
fn output_path_cannot_overwrite_input() {
    let temp_dir = tempdir().unwrap();
    let input = temp_dir.path().join("source.md");
    fs::write(&input, "# Original document\n").unwrap();

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            input.to_str().unwrap(),
            "--output",
            input.to_str().unwrap(),
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "output path would overwrite the input file",
        ));

    assert_eq!(fs::read_to_string(input).unwrap(), "# Original document\n");
}

#[test]
fn keep_html_rejects_an_html_pdf_output_path() {
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("output.HTML");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--output",
            output.to_str().unwrap(),
            "--keep-html",
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "HTML debug path conflicts with the PDF output",
        ));

    assert!(!output.exists());
}

#[test]
fn keep_html_creates_output_parent_directory_before_browser_discovery() {
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("nested/out.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--output",
            output.to_str().unwrap(),
            "--keep-html",
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to start browser"));

    let html = temp_dir.path().join("nested/out.html");
    assert!(html.exists());
    assert!(fs::read_to_string(html)
        .unwrap()
        .contains("Markdown to PDF"));
}

#[test]
fn title_is_written_to_kept_html() {
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("nested/out.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--output",
            output.to_str().unwrap(),
            "--title",
            "Quarterly Report",
            "--keep-html",
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to start browser"));

    let html = temp_dir.path().join("nested/out.html");
    assert!(fs::read_to_string(html)
        .unwrap()
        .contains("<title>Quarterly Report</title>"));
}

#[test]
fn browser_smoke_plain_markdown() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("basic.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
        ])
        .assert()
        .success();

    let doc = pdf::load_pdf(&output).expect("generated PDF should be readable");
    assert_eq!(
        pdf::page_count(&doc),
        1,
        "plain Markdown should fit on one page"
    );
    assert!(
        pdf::contains_text(&doc, "Markdown to PDF").expect("text should be extractable"),
        "PDF should contain the document heading"
    );
    assert!(
        pdf::contains_text(&doc, "This fixture checks plain Markdown rendering.")
            .expect("text should be extractable"),
        "PDF should contain the document body"
    );
}

#[test]
fn browser_smoke_valid_mermaid() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("mermaid.pdf");

    let mut command = Command::cargo_bin("md-to-pdf").unwrap();
    command.args([
        "fixtures/mermaid-flowchart.md",
        "--output",
        output.to_str().unwrap(),
        "--browser",
        &browser,
        "--virtual-time-budget",
        "15000",
    ]);
    if let Some(bundle) = mermaid_bundle() {
        command.args(["--mermaid-js", &bundle]);
    }
    command.assert().success();

    let doc = pdf::load_pdf(&output).expect("generated PDF should be readable");
    assert_eq!(
        pdf::page_count(&doc),
        1,
        "Mermaid fixture should fit on one page"
    );
    for label in ["Markdown", "HTML", "Mermaid", "PDF"] {
        assert!(
            pdf::contains_text(&doc, label).expect("text should be extractable"),
            "PDF should contain rendered Mermaid node label: {label}"
        );
    }
}

#[test]
fn browser_smoke_invalid_mermaid_fails() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("invalid.pdf");

    let mut command = Command::cargo_bin("md-to-pdf").unwrap();
    command.args([
        "fixtures/invalid-mermaid.md",
        "--output",
        output.to_str().unwrap(),
        "--browser",
        &browser,
        "--virtual-time-budget",
        "15000",
    ]);
    if let Some(bundle) = mermaid_bundle() {
        command.args(["--mermaid-js", &bundle]);
    }
    command
        .assert()
        .failure()
        .stderr(predicate::str::contains("Mermaid render failed"));
}

#[test]
fn browser_smoke_letter_page_size() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("letter.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
            "--page-size",
            "Letter",
        ])
        .assert()
        .success();

    let doc = pdf::load_pdf(&output).expect("generated PDF should be readable");
    let (width_mm, height_mm) = pdf::page_size_mm(&doc, 1).expect("page size should be readable");
    assert!(
        (width_mm - 215.9).abs() < 1.0,
        "Letter width should be ~215.9 mm, got {width_mm}"
    );
    assert!(
        (height_mm - 279.4).abs() < 1.0,
        "Letter height should be ~279.4 mm, got {height_mm}"
    );
}

#[test]
fn browser_smoke_raw_html_is_escaped_by_default() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let input = temp_dir.path().join("unsafe.md");
    let output = temp_dir.path().join("safe.pdf");
    fs::write(&input, "# Safe\n\n<script>alert(1)</script>\n").unwrap();

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
        ])
        .assert()
        .success();

    let doc = pdf::load_pdf(&output).expect("generated PDF should be readable");
    assert!(
        pdf::contains_text(&doc, "<script>alert(1)</script>").expect("text should be extractable"),
        "raw HTML should be escaped to literal text"
    );
}

fn smoke_browser() -> Option<String> {
    std::env::var("MD_TO_PDF_BROWSER").ok()
}

fn mermaid_bundle() -> Option<String> {
    std::env::var("MD_TO_PDF_MERMAID_JS").ok()
}
