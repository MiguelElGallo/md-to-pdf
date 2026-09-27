use html_escape::{encode_double_quoted_attribute, encode_text};

pub const DEFAULT_MERMAID_URL: &str =
    "https://cdn.jsdelivr.net/npm/mermaid@11.12.0/dist/mermaid.esm.min.mjs";

/// A validated CSS `@page` size such as `A4`, `Letter landscape`, or
/// `210mm 297mm`. Construction rejects characters that could escape the CSS
/// declaration, so every rendering entry point shares the same protection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageSize(String);

impl PageSize {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self("A4".to_string())
    }
}

impl std::fmt::Display for PageSize {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::str::FromStr for PageSize {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_page_size(value)
    }
}

pub fn parse_page_size(value: &str) -> Result<PageSize, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("page size must not be empty".to_string());
    }
    if value.len() > 64
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, ' ' | '.' | '-')
        })
    {
        return Err(format!(
            "invalid page size {value:?}; use a CSS page size such as A4, Letter, \"A4 landscape\", or \"210mm 297mm\""
        ));
    }
    Ok(PageSize(value.to_string()))
}

#[derive(Debug, Clone)]
pub enum MermaidSource {
    EsModuleUrl(String),
    InlineScript(String),
}

#[derive(Debug, Clone)]
pub struct DocumentOptions {
    pub title: String,
    pub base_href: Option<String>,
    pub page_size: PageSize,
    pub custom_css: Option<String>,
    pub mermaid_source: MermaidSource,
    pub allow_remote_assets: bool,
}

pub fn render_document(body: &str, options: &DocumentOptions) -> String {
    let title = encode_text(&options.title);
    let page_size = options.page_size.as_str();
    let has_mermaid = body.contains("class=\"mermaid\"");
    let mermaid_status = if has_mermaid { "pending" } else { "ready" };
    let base = options
        .base_href
        .as_ref()
        .map(|href| format!("<base href=\"{}\">", encode_double_quoted_attribute(href)))
        .unwrap_or_default();
    let custom_css = options
        .custom_css
        .as_ref()
        .map(|css| format!("\n<style>\n{}\n</style>", css))
        .unwrap_or_default();
    let mermaid_loader = if has_mermaid {
        mermaid_loader(&options.mermaid_source)
    } else {
        String::new()
    };
    let content_security_policy = if options.allow_remote_assets {
        "default-src 'none'; base-uri 'self'; img-src data: file: http: https:; style-src 'unsafe-inline' http: https:; font-src data: file: http: https:; media-src data: file: http: https:; script-src 'unsafe-inline' http: https:; connect-src http: https:; object-src 'none'; frame-src http: https:; worker-src blob: http: https:"
    } else {
        "default-src 'none'; base-uri 'self'; img-src data: file:; style-src 'unsafe-inline'; font-src data: file:; media-src data: file:; script-src 'unsafe-inline' https://cdn.jsdelivr.net; connect-src https://cdn.jsdelivr.net; object-src 'none'; frame-src 'none'; worker-src 'none'"
    };

    format!(
        r#"<!doctype html>
    <html lang="en" data-mermaid-status="{mermaid_status}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="{content_security_policy}">
{base}
<title>{title}</title>
<style>
@page {{ size: {page_size}; margin: 20mm; }}
:root {{ color-scheme: light; }}
body {{
  box-sizing: border-box;
  color: #1f2933;
  font-family: ui-serif, Georgia, Cambria, "Times New Roman", Times, serif;
  font-size: 12pt;
  line-height: 1.55;
  margin: 0 auto;
  max-width: 820px;
  padding: 24px;
}}
h1, h2, h3, h4 {{
  color: #111827;
  font-family: ui-sans-serif, system-ui, sans-serif;
  line-height: 1.2;
  margin: 1.4em 0 0.45em;
}}
h1 {{ font-size: 28pt; }}
h2 {{ font-size: 20pt; }}
h3 {{ font-size: 16pt; }}
a {{ color: #0f766e; }}
code {{
  background: #f3f4f6;
  border-radius: 4px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
  font-size: 0.9em;
  padding: 0.1em 0.25em;
}}
pre {{
  background: #f8fafc;
  border: 1px solid #d9e2ec;
  border-radius: 6px;
  overflow-x: auto;
  padding: 14px;
}}
pre code {{ background: transparent; padding: 0; }}
table {{ border-collapse: collapse; width: 100%; }}
th, td {{ border: 1px solid #cbd5e1; padding: 6px 8px; }}
blockquote {{ border-left: 4px solid #94a3b8; color: #475569; margin-left: 0; padding-left: 14px; }}
img, svg {{ max-width: 100%; }}
.mermaid {{
  background: #ffffff;
  border: 1px solid #d9e2ec;
  border-radius: 6px;
  display: flex;
  justify-content: center;
  margin: 18px 0;
  overflow: visible;
  padding: 16px;
}}
@media print {{
  body {{ max-width: none; padding: 0; }}
  pre, blockquote, table, .mermaid {{ break-inside: avoid; }}
}}
</style>{custom_css}
</head>
<body>
{body}
{mermaid_loader}
</body>
</html>
"#
    )
}

fn mermaid_loader(source: &MermaidSource) -> String {
    match source {
        MermaidSource::EsModuleUrl(url) => format!(
            r#"{}
<script type="module">
let mermaidModule;
try {{
  mermaidModule = await import("{}");
}} catch (error) {{
  reportMermaidError(error, true);
}}
if (mermaidModule) {{
  try {{
    await renderMermaid(mermaidModule.default);
  }} catch (error) {{
    reportMermaidError(error);
  }}
}}
</script>"#,
            render_mermaid_function(),
            encode_double_quoted_attribute(url)
        ),
        MermaidSource::InlineScript(script) => format!(
            r#"{}
<script>
{}
</script>
<script>
(async () => {{
  try {{
    await renderMermaid(window.mermaid);
  }} catch (error) {{
    reportMermaidError(error);
  }}
}})();
</script>"#,
            render_mermaid_function(),
            script
        ),
    }
}

fn render_mermaid_function() -> &'static str {
    r##"<script>
async function renderMermaid(mermaid) {
  if (!mermaid) {
    throw new Error("Mermaid runtime did not load");
  }
  mermaid.initialize({ startOnLoad: false, securityLevel: "strict" });
  await mermaid.run({ querySelector: ".mermaid" });
  document.documentElement.dataset.mermaidStatus = "ready";
  window.__MD_TO_PDF_READY = true;
}

// Mermaid lazily imports diagram chunks while rendering, so network failures
// can also surface from mermaid.run().
function isMermaidLoadError(messageText) {
  return /Failed to fetch dynamically imported module|error loading dynamically imported module|Importing a module script failed/i.test(messageText);
}

function reportMermaidError(error, isLoadError = false) {
  const messageText = error && error.message ? error.message : String(error);
  document.documentElement.dataset.mermaidStatus =
    isLoadError || isMermaidLoadError(messageText) ? "load-error" : "error";
  window.__MD_TO_PDF_READY = false;
  window.__MD_TO_PDF_ERROR = messageText;

  const message = document.createElement("pre");
  message.style.color = "#b91c1c";
  message.textContent = `Mermaid render failed: ${messageText}`;
  document.body.appendChild(message);
}
</script>"##
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_css_page_sizes() {
        for size in [
            "A4",
            "Letter",
            "A4 landscape",
            "210mm 297mm",
            "8.5in 11in",
            " Legal ",
        ] {
            assert_eq!(parse_page_size(size).unwrap().as_str(), size.trim());
        }
    }

    #[test]
    fn rejects_page_sizes_that_could_inject_css() {
        for size in [
            "",
            "A4; margin: 0",
            "A4 } body { display: none",
            "A4</style>",
            "A4\nLetter",
        ] {
            assert!(
                parse_page_size(size).is_err(),
                "{size:?} should be rejected"
            );
        }
    }

    #[test]
    fn renders_base_href_and_mermaid_loader() {
        let html = render_document(
            "<h1>Doc</h1><pre class=\"mermaid\">graph TD\nA --&gt; B</pre>",
            &DocumentOptions {
                title: "Doc".to_string(),
                base_href: Some("file:///tmp/docs/".to_string()),
                page_size: PageSize::default(),
                custom_css: None,
                mermaid_source: MermaidSource::EsModuleUrl(DEFAULT_MERMAID_URL.to_string()),
                allow_remote_assets: false,
            },
        );

        assert!(html.contains("<base href=\"file:///tmp/docs/\">"));
        assert!(html.contains("data-mermaid-status=\"pending\""));
        assert!(html.contains(DEFAULT_MERMAID_URL));
        assert!(html.contains("img-src data: file:"));
        assert!(!html.contains("img-src data: file: http: https:"));
    }

    #[test]
    fn skips_mermaid_loader_when_document_has_no_mermaid_blocks() {
        let html = render_document(
            "<h1>Doc</h1>",
            &DocumentOptions {
                title: "Doc".to_string(),
                base_href: None,
                page_size: PageSize::default(),
                custom_css: None,
                mermaid_source: MermaidSource::EsModuleUrl(DEFAULT_MERMAID_URL.to_string()),
                allow_remote_assets: false,
            },
        );

        assert!(html.contains("data-mermaid-status=\"ready\""));
        assert!(!html.contains(DEFAULT_MERMAID_URL));
    }

    #[test]
    fn remote_assets_require_an_explicit_opt_in() {
        let html = render_document(
            r#"<img src="http://127.0.0.1/private">"#,
            &DocumentOptions {
                title: "Doc".to_string(),
                base_href: None,
                page_size: PageSize::default(),
                custom_css: Some("body { background: url(http://127.0.0.1/private); }".to_string()),
                mermaid_source: MermaidSource::InlineScript(String::new()),
                allow_remote_assets: true,
            },
        );

        assert!(html.contains("img-src data: file: http: https:"));
        assert!(html.contains("style-src 'unsafe-inline' http: https:"));
    }
}
