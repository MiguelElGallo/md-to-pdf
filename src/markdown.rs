use html_escape::encode_text;
use pulldown_cmark::{html, CodeBlockKind, CowStr, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashSet;

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

    assign_heading_ids(&mut events);

    let mut body = String::new();
    html::push_html(&mut body, events.into_iter());
    body
}

/// Gives headings without an explicit `{#id}` a GitHub-style slug so that
/// in-document links such as `[Usage](#usage)` resolve in the generated PDF.
fn assign_heading_ids(events: &mut [Event<'_>]) {
    let mut used: HashSet<String> = events
        .iter()
        .filter_map(|event| match event {
            Event::Start(Tag::Heading { id: Some(id), .. }) => Some(id.to_string()),
            // pulldown-cmark uses footnote labels as element ids.
            Event::Start(Tag::FootnoteDefinition(label)) | Event::FootnoteReference(label) => {
                Some(label.to_string())
            }
            _ => None,
        })
        .collect();

    let mut index = 0;
    while index < events.len() {
        if !matches!(events[index], Event::Start(Tag::Heading { id: None, .. })) {
            index += 1;
            continue;
        }

        let mut text = String::new();
        let mut end = index + 1;
        while end < events.len() && !matches!(events[end], Event::End(TagEnd::Heading(_))) {
            match &events[end] {
                Event::Text(value) | Event::Code(value) => text.push_str(value),
                Event::SoftBreak | Event::HardBreak => text.push(' '),
                _ => {}
            }
            end += 1;
        }

        let slug = unique_slug(&slugify(&text), &mut used);
        if let Event::Start(Tag::Heading { id, .. }) = &mut events[index] {
            *id = Some(CowStr::from(slug));
        }
        index = end;
    }
}

fn slugify(text: &str) -> String {
    let slug: String = text
        .trim()
        .chars()
        .filter_map(|character| {
            if character.is_alphanumeric() || character == '_' || character == '-' {
                Some(character.to_lowercase().collect::<String>())
            } else if character.is_whitespace() {
                Some("-".to_string())
            } else {
                None
            }
        })
        .collect();

    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

fn unique_slug(base: &str, used: &mut HashSet<String>) -> String {
    if used.insert(base.to_string()) {
        return base.to_string();
    }

    let mut suffix = 1;
    loop {
        let candidate = format!("{base}-{suffix}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
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

        assert!(html.contains("<h1 id=\"title\">Title</h1>"));
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

    #[test]
    fn assigns_github_style_heading_ids() {
        let html = markdown_to_body(
            "# Getting Started!\n\n## `cargo` & Rust\n\n## Getting Started\n\n## Custom {#getting-started-1}\n\n## Überblick",
            &HtmlOptions::default(),
        );

        assert!(html.contains("<h1 id=\"getting-started\">"));
        assert!(html.contains("<h2 id=\"cargo--rust\">"));
        assert!(html.contains("<h2 id=\"getting-started-2\">"));
        assert!(html.contains("<h2 id=\"getting-started-1\">"));
        assert!(html.contains("<h2 id=\"überblick\">"));
    }

    #[test]
    fn separates_words_across_setext_heading_lines() {
        let html = markdown_to_body("Getting\nStarted\n=======", &HtmlOptions::default());

        assert!(html.contains("<h1 id=\"getting-started\">"));
    }

    #[test]
    fn heading_ids_do_not_collide_with_footnote_ids() {
        let html = markdown_to_body(
            "# Note\n\nText[^note].\n\n[^note]: A footnote.",
            &HtmlOptions::default(),
        );

        assert!(html.contains("<h1 id=\"note-1\">"));
        assert!(html.contains("id=\"note\""));
        assert!(html.contains("href=\"#note\""));
    }

    #[test]
    fn falls_back_to_section_for_empty_heading_slugs() {
        let html = markdown_to_body("# !!!\n\n# ???", &HtmlOptions::default());

        assert!(html.contains("<h1 id=\"section\">"));
        assert!(html.contains("<h1 id=\"section-1\">"));
    }
}
