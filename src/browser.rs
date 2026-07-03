use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use camino::{Utf8Path, Utf8PathBuf};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fs;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tungstenite::{connect, Message, WebSocket};
use url::Url;

#[derive(Debug, Clone)]
pub struct BrowserOptions {
    pub browser: Option<Utf8PathBuf>,
    pub virtual_time_budget_ms: u64,
    pub allow_local_files: bool,
}

pub fn print_to_pdf(
    html_path: &Utf8Path,
    pdf_path: &Utf8Path,
    options: &BrowserOptions,
) -> Result<()> {
    let browser = match &options.browser {
        Some(path) => path.clone(),
        None => discover_browser()?,
    };

    if let Some(parent) = pdf_path.parent() {
        if !parent.as_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create output directory {parent}"))?;
        }
    }

    let html_url = file_url(html_path)?;
    let mut browser_process = BrowserProcess::launch(&browser, options)?;
    let page = browser_process.create_page(html_url.as_str())?;
    let mut client = CdpClient::connect(&page.web_socket_debugger_url)?;

    client.send("Page.enable", json!({}))?;
    client.send("Runtime.enable", json!({}))?;
    wait_for_mermaid(
        &mut client,
        Duration::from_millis(options.virtual_time_budget_ms),
    )?;

    let pdf = client.send(
        "Page.printToPDF",
        json!({
            "printBackground": true,
            "preferCSSPageSize": true
        }),
    )?;
    let data = pdf
        .get("data")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("browser did not return PDF data"))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .context("failed to decode browser PDF data")?;
    if bytes.is_empty() {
        bail!("browser returned an empty PDF");
    }

    fs::write(pdf_path, bytes).with_context(|| format!("failed to write {pdf_path}"))?;
    Ok(())
}

struct BrowserProcess {
    child: Child,
    _profile: TempDir,
    port: u16,
}

impl BrowserProcess {
    fn launch(browser: &Utf8Path, options: &BrowserOptions) -> Result<Self> {
        let profile = tempfile::tempdir().context("failed to create browser profile")?;
        let mut command = Command::new(browser.as_std_path());
        command
            .arg("--headless=new")
            .arg("--disable-gpu")
            .arg("--disable-extensions")
            .arg("--no-first-run")
            .arg("--no-default-browser-check")
            .arg("--remote-debugging-port=0")
            .arg(format!(
                "--user-data-dir={}",
                Utf8Path::from_path(profile.path()).unwrap()
            ))
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if options.allow_local_files {
            command.arg("--allow-file-access-from-files");
        }

        command.arg("about:blank");

        let child = command
            .spawn()
            .with_context(|| format!("failed to start browser at {browser}"))?;
        let port = read_devtools_port(profile.path(), options.virtual_time_budget_ms)?;

        Ok(Self {
            child,
            _profile: profile,
            port,
        })
    }

    fn create_page(&mut self, url: &str) -> Result<PageTarget> {
        let encoded_url = url::form_urlencoded::byte_serialize(url.as_bytes()).collect::<String>();
        let endpoint = format!("http://127.0.0.1:{}/json/new?{}", self.port, encoded_url);
        let response = ureq::put(&endpoint)
            .call()
            .with_context(|| format!("failed to create browser page at {endpoint}"))?
            .into_string()
            .context("failed to read browser page response")?;
        serde_json::from_str(&response).context("failed to parse browser page response")
    }
}

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[derive(Debug, Deserialize)]
struct PageTarget {
    #[serde(rename = "webSocketDebuggerUrl")]
    web_socket_debugger_url: String,
}

struct CdpClient {
    socket: WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    next_id: u64,
}

impl CdpClient {
    fn connect(url: &str) -> Result<Self> {
        let (socket, _) = connect(url).with_context(|| format!("failed to connect to {url}"))?;
        Ok(Self { socket, next_id: 1 })
    }

    fn send(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let message = json!({
            "id": id,
            "method": method,
            "params": params
        });
        self.socket
            .send(Message::Text(message.to_string()))
            .with_context(|| format!("failed to send browser command {method}"))?;

        loop {
            let message = self
                .socket
                .read()
                .with_context(|| format!("failed while waiting for browser command {method}"))?;
            let Message::Text(text) = message else {
                continue;
            };
            let value: Value =
                serde_json::from_str(&text).context("failed to parse browser event")?;
            if value.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = value.get("error") {
                bail!("browser command {method} failed: {error}");
            }
            return Ok(value.get("result").cloned().unwrap_or(Value::Null));
        }
    }
}

fn wait_for_mermaid(client: &mut CdpClient, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let value = evaluate_json(
            client,
            r#"(() => ({
  readyState: document.readyState,
  status: document.documentElement.dataset.mermaidStatus || "missing",
  error: window.__MD_TO_PDF_ERROR || ""
}))()"#,
        )?;
        let ready_state = value
            .get("readyState")
            .and_then(Value::as_str)
            .unwrap_or("");
        let status = value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("missing");
        let error = value.get("error").and_then(Value::as_str).unwrap_or("");

        if status == "error" {
            bail!("Mermaid render failed: {error}");
        }
        if status == "ready" && ready_state == "complete" {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(50));
    }

    bail!("timed out waiting for Mermaid rendering")
}

fn evaluate_json(client: &mut CdpClient, expression: &str) -> Result<Value> {
    let result = client.send(
        "Runtime.evaluate",
        json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true
        }),
    )?;
    result
        .get("result")
        .and_then(|result| result.get("value"))
        .cloned()
        .ok_or_else(|| anyhow!("browser evaluation did not return a value"))
}

fn read_devtools_port(profile: &std::path::Path, timeout_ms: u64) -> Result<u16> {
    let path = profile.join("DevToolsActivePort");
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    while Instant::now() < deadline {
        if let Ok(contents) = fs::read_to_string(&path) {
            let port = contents
                .lines()
                .next()
                .ok_or_else(|| anyhow!("DevToolsActivePort did not contain a port"))?
                .parse::<u16>()
                .context("DevToolsActivePort contained an invalid port")?;
            return Ok(port);
        }
        thread::sleep(Duration::from_millis(50));
    }

    bail!("timed out waiting for browser DevTools port")
}

pub fn discover_browser() -> Result<Utf8PathBuf> {
    if let Ok(path) = std::env::var("MD_TO_PDF_BROWSER") {
        let path = Utf8PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    for candidate in browser_command_names() {
        if let Ok(path) = which::which(candidate) {
            return Utf8PathBuf::from_path_buf(path)
                .map_err(|path| anyhow!("browser path is not valid UTF-8: {}", path.display()));
        }
    }

    for candidate in browser_app_paths() {
        let path = Utf8PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    bail!("could not find Chrome, Chromium, or Edge; pass --browser or set MD_TO_PDF_BROWSER")
}

pub fn file_url(path: &Utf8Path) -> Result<Url> {
    let canonical = path
        .canonicalize_utf8()
        .with_context(|| format!("failed to resolve {path}"))?;
    Url::from_file_path(canonical.as_std_path())
        .map_err(|_| anyhow!("failed to build file URL for {path}"))
}

fn browser_command_names() -> &'static [&'static str] {
    &[
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
        "microsoft-edge",
        "msedge",
        "chrome",
    ]
}

fn browser_app_paths() -> &'static [&'static str] {
    &[
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
    ]
}
