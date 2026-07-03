use html_escape::encode_text;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, Default)]
pub struct HtmlOptions {
    pub allow_html: bool,
}

pub fn markdown_to_body(markdown: &str, options: &HtmlOptions) -> String {
    let parser = Parser::new_ext(markdown, markdown_options());
    let mut events = Vec::new();
    let mut in_mermaid_block = false;

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language)))
                if is_mermaid(&language) =>
            {
                in_mermaid_block = true;
                events.push(Event::Html(CowStr::from("<pre class=\"mermaid\">")));
            }
            Event::End(TagEnd::CodeBlock) if in_mermaid_block => {
                in_mermaid_block = false;
                events.push(Event::Html(CowStr::from("</pre>")));
            }
            Event::Text(text) if in_mermaid_block => {
                events.push(Event::Html(CowStr::from(encode_text(&text).into_owned())));
            }
            Event::Html(html) | Event::InlineHtml(html) if !options.allow_html => {
                events.push(Event::Text(html));
            }
            other => events.push(other),
        }
    }

    let mut body = String::new();
    html::push_html(&mut body, events.into_iter());
    body
}

fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
}

fn is_mermaid(language: &str) -> bool {
    language
        .split_whitespace()
        .next()
        .is_some_and(|name| name.eq_ignore_ascii_case("mermaid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_basic_markdown() {
        let html = markdown_to_body("# Title\n\n- one\n- two", &HtmlOptions::default());

        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<li>one</li>"));
    }

    #[test]
    fn converts_mermaid_fences_to_mermaid_blocks() {
        let html = markdown_to_body(
            "```mermaid\ngraph TD\n  A --> B\n```",
            &HtmlOptions::default(),
        );

        assert!(html.contains("<pre class=\"mermaid\">"));
        assert!(html.contains("graph TD"));
        assert!(html.contains("A --&gt; B"));
        assert!(!html.contains("language-mermaid"));
    }

    #[test]
    fn preserves_non_mermaid_code_fences() {
        let html = markdown_to_body("```rust\nfn main() {}\n```", &HtmlOptions::default());

        assert!(html.contains("language-rust"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn escapes_raw_html_by_default() {
        let html = markdown_to_body("<script>alert(1)</script>", &HtmlOptions::default());

        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn can_allow_raw_html() {
        let html = markdown_to_body(
            "<section>trusted</section>",
            &HtmlOptions { allow_html: true },
        );

        assert!(html.contains("<section>trusted</section>"));
    }
}
