use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};
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
        .stdout(predicate::str::contains("--allow-remote-assets"))
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
fn custom_mermaid_url_requires_remote_asset_opt_in() {
    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/mermaid-flowchart.md",
            "--mermaid-url",
            "https://example.com/mermaid.mjs",
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "a custom Mermaid URL requires --allow-remote-assets",
        ));
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

#[test]
fn browser_blocks_document_remote_requests_by_default() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser network test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/private.png", listener.local_addr().unwrap());
    let temp_dir = tempdir().unwrap();
    let input = temp_dir.path().join("remote.md");
    let css = temp_dir.path().join("remote.css");
    let output = temp_dir.path().join("remote.pdf");
    fs::write(&input, format!("# Remote assets\n\n![private]({url})\n")).unwrap();
    fs::write(
        &css,
        format!("body {{ background-image: url('{url}'); }}\n"),
    )
    .unwrap();

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            input.to_str().unwrap(),
            "--css",
            css.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
        ])
        .assert()
        .success();

    assert!(matches!(
        listener.accept(),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
    ));
}

#[test]
fn browser_remote_asset_opt_in_preserves_network_images() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser network test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/asset.png", listener.local_addr().unwrap());
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0_u8; 1024];
                    let _ = stream.read(&mut request);
                    stream
                        .write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n")
                        .unwrap();
                    return true;
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return false;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
                Err(error) => panic!("asset server failed: {error}"),
            }
        }
    });
    let temp_dir = tempdir().unwrap();
    let input = temp_dir.path().join("remote.md");
    let output = temp_dir.path().join("remote.pdf");
    fs::write(&input, format!("# Remote asset\n\n![remote]({url})\n")).unwrap();

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--allow-remote-assets",
            "--browser",
            &browser,
        ])
        .assert()
        .success();

    assert!(
        server.join().unwrap(),
        "browser did not request remote asset"
    );
}

fn smoke_browser() -> Option<String> {
    std::env::var("MD_TO_PDF_BROWSER").ok()
}
