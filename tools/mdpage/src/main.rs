//! Render a Markdown file into an HTML page template: `mdpage <in.md> <template.html> <out.html>`.
//!
//! The site's How-it-works page is `HOW-IT-WORKS.md`, rendered at build time so the page
//! and the file can't drift apart. The template holds `<!-- TITLE -->` (replaced by the
//! document's first heading) and `<!-- CONTENT -->` (replaced by the rendered body).

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let [input, template, output] = args.as_slice() else {
        eprintln!("usage: mdpage <in.md> <template.html> <out.html>");
        process::exit(2);
    };
    let read = |path: &str| {
        fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("mdpage: can't read {path}: {e}");
            process::exit(1)
        })
    };
    let page = render(&read(input), &read(template));
    if let Err(e) = fs::write(output, page) {
        eprintln!("mdpage: can't write {output}: {e}");
        process::exit(1);
    }
}

/// Fill `template` with the rendered `markdown` and its first top-level heading.
fn render(markdown: &str, template: &str) -> String {
    let parser = Parser::new_ext(
        markdown,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH,
    );
    let mut body = String::new();
    html::push_html(&mut body, parser);
    template
        .replace("<!-- TITLE -->", &escape(&title(markdown)))
        .replace("<!-- CONTENT -->", &body)
}

/// The text of the first `#` heading, or a fallback.
fn title(markdown: &str) -> String {
    let mut inside = false;
    let mut text = String::new();
    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => inside = true,
            Event::End(TagEnd::Heading(HeadingLevel::H1)) if inside => return text,
            Event::Text(t) | Event::Code(t) if inside => text.push_str(&t),
            _ => {}
        }
    }
    "Cellular automata".to_string()
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page's title comes from the document's first heading, and the body replaces the
    /// placeholder; code blocks must come through as `<pre><code>`, which the doc relies on.
    #[test]
    fn renders_title_and_body() {
        let md = "# How it works\n\nSome `code`.\n\n```rust\nlet x = 1;\n```\n";
        let page = render(
            md,
            "<title><!-- TITLE --></title><main><!-- CONTENT --></main>",
        );
        assert!(page.starts_with("<title>How it works</title>"));
        assert!(page.contains("<h1>How it works</h1>"));
        assert!(page.contains("<code>code</code>"));
        assert!(page.contains("<pre><code class=\"language-rust\">let x = 1;"));
    }

    /// Characters special in HTML must not break the title, and a document with no
    /// heading still gets one.
    #[test]
    fn titles_are_escaped_and_never_empty() {
        assert_eq!(escape(&title("# A < B & C\n")), "A &lt; B &amp; C");
        assert_eq!(title("no heading here"), "Cellular automata");
    }
}
