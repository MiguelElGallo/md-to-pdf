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
        .stdout(predicate::str::contains("--browser"))
        .stdout(predicate::str::contains("Per-attempt wall-clock timeout"))
        .stdout(predicate::str::contains("Not Chromium virtual time"));
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
fn output_paths_reject_hard_link_aliases() {
    for collision in ["input-pdf", "input-html", "pdf-html"] {
        let temp_dir = tempdir().unwrap();
        let input = temp_dir.path().join("source.md");
        let output = temp_dir.path().join("out.pdf");
        let html = output.with_extension("html");
        fs::write(&input, "# Original document\n").unwrap();
        let expected_error = match collision {
            "input-pdf" => {
                fs::hard_link(&input, &output).unwrap();
                "output path would overwrite the input file"
            }
            "input-html" => {
                fs::hard_link(&input, &html).unwrap();
                "HTML debug path would overwrite the input file"
            }
            "pdf-html" => {
                fs::write(&output, "Original PDF").unwrap();
                fs::hard_link(&output, &html).unwrap();
                "HTML debug path conflicts with the PDF output"
            }
            _ => unreachable!(),
        };

        Command::cargo_bin("md-to-pdf")
            .unwrap()
            .args([
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--keep-html",
                "--browser",
                "/definitely/not/a/browser",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains(expected_error));

        assert_eq!(fs::read_to_string(&input).unwrap(), "# Original document\n");
        if collision == "pdf-html" {
            assert_eq!(fs::read_to_string(&output).unwrap(), "Original PDF");
            assert_eq!(fs::read_to_string(&html).unwrap(), "Original PDF");
        }
    }
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

#[cfg(unix)]
#[test]
fn output_paths_reject_existing_and_dangling_symlinks() {
    for keep_html in [false, true] {
        for dangling in [false, true] {
            let temp_dir = tempdir().unwrap();
            let input = temp_dir.path().join("source.md");
            let output = temp_dir.path().join("out.pdf");
            let target = temp_dir.path().join("target");
            let link = if keep_html {
                output.with_extension("html")
            } else {
                output.clone()
            };
            fs::write(&input, "# Original document\n").unwrap();
            if !dangling {
                fs::write(&target, "Original target").unwrap();
            }
            std::os::unix::fs::symlink(&target, &link).unwrap();

            let mut command = Command::cargo_bin("md-to-pdf").unwrap();
            command.args([
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--browser",
                "/definitely/not/a/browser",
            ]);
            if keep_html {
                command.arg("--keep-html");
            }
            command
                .assert()
                .failure()
                .stderr(predicate::str::contains("output path is a symlink"));

            assert!(fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink());
            assert_eq!(fs::read_to_string(&input).unwrap(), "# Original document\n");
            if dangling {
                assert!(!target.exists());
            } else {
                assert_eq!(fs::read_to_string(&target).unwrap(), "Original target");
            }
        }
    }
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

#[cfg(unix)]
#[test]
fn output_paths_reject_fifos_without_blocking() {
    for keep_html in [false, true] {
        for use_symlink in [false, true] {
            let directory = tempdir().unwrap();
            let input = directory.path().join("source.md");
            let output = directory.path().join("out.pdf");
            let destination = if keep_html {
                output.with_extension("html")
            } else {
                output.clone()
            };
            let fifo = directory.path().join("pipe");
            fs::write(&input, "# Original document\n").unwrap();
            let fifo_path = if use_symlink { &fifo } else { &destination };
            assert!(std::process::Command::new("mkfifo")
                .arg(fifo_path)
                .status()
                .unwrap()
                .success());
            if use_symlink {
                std::os::unix::fs::symlink(&fifo, &destination).unwrap();
            }

            let mut command = Command::cargo_bin("md-to-pdf").unwrap();
            command.timeout(Duration::from_secs(5)).args([
                input.to_str().unwrap(),
                "--output",
                output.to_str().unwrap(),
                "--browser",
                "/definitely/not/a/browser",
            ]);
            if keep_html {
                command.arg("--keep-html");
            }
            command
                .assert()
                .failure()
                .stderr(predicate::str::contains(if use_symlink {
                    "output path is a symlink"
                } else {
                    "output path is not a regular file"
                }));
            assert_eq!(fs::read_to_string(&input).unwrap(), "# Original document\n");
        }
    }
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
    fs::write(&output, "Previous PDF").unwrap();

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

    assert!(fs::read(output).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn browser_smoke_headings_produce_outline_and_internal_links() {
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("headings.pdf");

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/headings.md",
            "--output",
            output.to_str().unwrap(),
            "--browser",
            &browser,
        ])
        .assert()
        .success();

    let pdf = fs::read(output).unwrap();
    let contains =
        |haystack: &[u8], needle: &[u8]| haystack.windows(needle.len()).any(|w| w == needle);
    assert!(
        contains(&pdf, b"/Outlines"),
        "PDF should include a document outline"
    );
    let has_internal_link_annotation =
        String::from_utf8_lossy(&pdf).split("endobj").any(|object| {
            object.contains("/Type /Annot")
                && object.contains("/Subtype /Link")
                && object.contains("/Dest /results")
                && !object.contains("/URI")
        });
    assert!(
        has_internal_link_annotation,
        "the [results](#results) Markdown link should become a link annotation pointing at /results"
    );
    assert!(
        contains(&pdf, b"/StructTreeRoot"),
        "PDF should be tagged with a structure tree"
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
    fs::write(&output, "Previous PDF").unwrap();

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
            "--keep-html",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Mermaid render failed: Parse error",
        ))
        .stderr(predicate::str::contains("Mermaid failed to load after").not());

    assert_eq!(fs::read_to_string(&output).unwrap(), "Previous PDF");
    assert!(output.with_extension("html").exists());
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

#[test]
fn invalid_page_size_fails_before_browser_discovery() {
    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            "fixtures/basic.md",
            "--page-size",
            "A4; margin: 0",
            "--browser",
            "/definitely/not/a/browser",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid page size"));
}

#[test]
fn bare_input_file_name_resolves_relative_to_current_directory() {
    let temp_dir = tempdir().unwrap();
    fs::write(temp_dir.path().join("report.md"), "# Report\n").unwrap();

    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .current_dir(temp_dir.path())
        .args(["report.md", "--browser", "/definitely/not/a/browser"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to start browser"))
        .stderr(predicate::str::contains("failed to resolve").not());
}

/// Serves a stand-in Mermaid ES module whose `run()` lazily imports a chunk,
/// like the real runtime. The entry module fails with HTTP 503 for the first
/// `entry_failures` requests and the chunk for the first `chunk_failures`.
/// Returns the module URL, a stop flag, and the entry and chunk request counters.
fn flaky_mermaid_server(
    entry_failures: usize,
    chunk_failures: usize,
) -> (
    String,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
    std::sync::Arc<std::sync::atomic::AtomicUsize>,
) {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/mermaid.mjs", listener.local_addr().unwrap());
    let stop = Arc::new(AtomicBool::new(false));
    let entry_requests = Arc::new(AtomicUsize::new(0));
    let chunk_requests = Arc::new(AtomicUsize::new(0));
    let (server_stop, server_entry, server_chunk) =
        (stop.clone(), entry_requests.clone(), chunk_requests.clone());
    thread::spawn(move || {
        while !server_stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut request = [0_u8; 2048];
                    let _ = stream.read(&mut request);
                    let request_line = String::from_utf8_lossy(&request)
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    let (counter, failures, body) = if request_line.contains("/mermaid.mjs ") {
                        (
                            &server_entry,
                            entry_failures,
                            r#"export default { initialize() {}, async run() { await import("./chunk.mjs"); if (document.querySelector(".mermaid").textContent.trim().endsWith("-->")) { throw new Error("Parse error on line 3"); } } };"#,
                        )
                    } else if request_line.contains("/chunk.mjs ") {
                        (&server_chunk, chunk_failures, "export const loaded = true;")
                    } else {
                        let _ = stream.write_all(
                            b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        );
                        continue;
                    };
                    let response = if counter.fetch_add(1, Ordering::SeqCst) < failures {
                        "HTTP/1.1 503 Service Unavailable\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                    } else {
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/javascript\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        )
                    };
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("Mermaid server failed: {error}"),
            }
        }
    });
    (url, stop, entry_requests, chunk_requests)
}

fn convert_with_mermaid_url(browser: &str, url: &str) -> assert_cmd::assert::Assert {
    convert_fixture_with_mermaid_url(browser, url, "fixtures/mermaid-flowchart.md")
}

fn convert_fixture_with_mermaid_url(
    browser: &str,
    url: &str,
    fixture: &str,
) -> assert_cmd::assert::Assert {
    let temp_dir = tempdir().unwrap();
    let output = temp_dir.path().join("retry.pdf");
    Command::cargo_bin("md-to-pdf")
        .unwrap()
        .args([
            fixture,
            "--output",
            output.to_str().unwrap(),
            "--browser",
            browser,
            "--allow-remote-assets",
            "--mermaid-url",
            url,
        ])
        .assert()
}

#[test]
fn browser_smoke_mermaid_load_failure_is_retried() {
    use std::sync::atomic::Ordering;
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let (url, stop, requests, chunks) = flaky_mermaid_server(1, 0);

    let started = Instant::now();
    convert_with_mermaid_url(&browser, &url)
        .success()
        .stderr(predicate::str::contains(
            "Mermaid failed to load: Failed to fetch dynamically imported module",
        ))
        .stderr(predicate::str::contains("Retrying in 1s (retry 1 of 4)..."))
        .stderr(predicate::str::contains("retry 2 of 4").not());
    stop.store(true, Ordering::SeqCst);

    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(chunks.load(Ordering::SeqCst), 1);
    assert!(started.elapsed() >= Duration::from_secs(1));
}

#[test]
fn browser_smoke_mermaid_chunk_failure_during_render_is_retried() {
    use std::sync::atomic::Ordering;
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let (url, stop, requests, chunks) = flaky_mermaid_server(0, 1);

    convert_with_mermaid_url(&browser, &url)
        .success()
        .stderr(predicate::str::contains(
            "Mermaid failed to load: Failed to fetch dynamically imported module",
        ))
        .stderr(predicate::str::contains("chunk.mjs"))
        .stderr(predicate::str::contains("Retrying in 1s (retry 1 of 4)..."))
        .stderr(predicate::str::contains("retry 2 of 4").not());
    stop.store(true, Ordering::SeqCst);

    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(chunks.load(Ordering::SeqCst), 2);
}

#[test]
fn browser_smoke_mermaid_load_failure_gives_up_after_all_retries() {
    use std::sync::atomic::Ordering;
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let (url, stop, requests, _) = flaky_mermaid_server(usize::MAX, 0);

    let started = Instant::now();
    convert_with_mermaid_url(&browser, &url)
        .failure()
        .stderr(predicate::str::contains("Retrying in 1s (retry 1 of 4)..."))
        .stderr(predicate::str::contains("Retrying in 3s (retry 2 of 4)..."))
        .stderr(predicate::str::contains("Retrying in 5s (retry 3 of 4)..."))
        .stderr(predicate::str::contains(
            "Retrying in 10s (retry 4 of 4)...",
        ))
        .stderr(predicate::str::contains(
            "Mermaid failed to load after 5 attempts",
        ));
    stop.store(true, Ordering::SeqCst);

    assert_eq!(requests.load(Ordering::SeqCst), 5);
    assert!(started.elapsed() >= Duration::from_secs(19));
}

#[test]
fn browser_smoke_mermaid_syntax_error_is_not_retried() {
    use std::sync::atomic::Ordering;
    let Some(browser) = smoke_browser() else {
        eprintln!("skipping browser smoke test; set MD_TO_PDF_BROWSER to enable it");
        return;
    };
    let (url, stop, requests, _) = flaky_mermaid_server(0, 0);

    convert_fixture_with_mermaid_url(&browser, &url, "fixtures/invalid-mermaid.md")
        .failure()
        .stderr(predicate::str::contains(
            "Mermaid render failed: Parse error on line 3",
        ))
        .stderr(predicate::str::contains("Retrying").not());
    stop.store(true, Ordering::SeqCst);

    assert_eq!(requests.load(Ordering::SeqCst), 1);
}
