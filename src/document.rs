use html_escape::{encode_double_quoted_attribute, encode_text};

pub const DEFAULT_MERMAID_URL: &str =
    "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs";

#[derive(Debug, Clone)]
pub enum MermaidSource {
    EsModuleUrl(String),
    InlineScript(String),
}

#[derive(Debug, Clone)]
pub struct DocumentOptions {
    pub title: String,
    pub base_href: Option<String>,
    pub page_size: String,
    pub custom_css: Option<String>,
    pub mermaid_source: MermaidSource,
}

pub fn render_document(body: &str, options: &DocumentOptions) -> String {
    let title = encode_text(&options.title);
    let page_size = encode_text(&options.page_size);
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

    format!(
        r#"<!doctype html>
    <html lang="en" data-mermaid-status="{mermaid_status}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
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
try {{
  const module = await import("{}");
  await renderMermaid(module.default);
}} catch (error) {{
  reportMermaidError(error);
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

function reportMermaidError(error) {
  const messageText = error && error.message ? error.message : String(error);
  document.documentElement.dataset.mermaidStatus = "error";
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
    fn renders_base_href_and_mermaid_loader() {
        let html = render_document(
            "<h1>Doc</h1><pre class=\"mermaid\">graph TD\nA --&gt; B</pre>",
            &DocumentOptions {
                title: "Doc".to_string(),
                base_href: Some("file:///tmp/docs/".to_string()),
                page_size: "A4".to_string(),
                custom_css: None,
                mermaid_source: MermaidSource::EsModuleUrl(DEFAULT_MERMAID_URL.to_string()),
            },
        );

        assert!(html.contains("<base href=\"file:///tmp/docs/\">"));
        assert!(html.contains("data-mermaid-status=\"pending\""));
        assert!(html.contains(DEFAULT_MERMAID_URL));
    }

    #[test]
    fn skips_mermaid_loader_when_document_has_no_mermaid_blocks() {
        let html = render_document(
            "<h1>Doc</h1>",
            &DocumentOptions {
                title: "Doc".to_string(),
                base_href: None,
                page_size: "A4".to_string(),
                custom_css: None,
                mermaid_source: MermaidSource::EsModuleUrl(DEFAULT_MERMAID_URL.to_string()),
            },
        );

        assert!(html.contains("data-mermaid-status=\"ready\""));
        assert!(!html.contains(DEFAULT_MERMAID_URL));
    }
}
