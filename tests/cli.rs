use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

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

    assert!(fs::metadata(output).unwrap().len() > 0);
}

#[test]
fn browser_smoke_valid_mermaid() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("mermaid.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/mermaid-flowchart.md",
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
            "--virtual-time-budget",
            "15000",
        ])
        .assert()
        .success();

    assert!(fs::metadata(output).unwrap().len() > 0);
}

#[test]
fn browser_smoke_invalid_mermaid_fails() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("invalid.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/invalid-mermaid.md",
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
            "--virtual-time-budget",
            "15000",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Mermaid render failed"));
}

fn smoke_browser() -> Option<String> {
    std::env::var("MD_TO_PDF_BROWSER").ok()
}
