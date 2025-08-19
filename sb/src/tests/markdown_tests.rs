#[cfg(test)]
mod tests {
    use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};

    #[test]
    fn test_markdown_parsing() {
        let markdown = "# Heading\n\nParagraph text\n\n- List item 1\n- List item 2";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        assert!(events.len() > 0);

        // Check for heading
        let has_heading = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Heading(HeadingLevel::H1, _, _))));
        assert!(has_heading);

        // Check for list
        let has_list = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::List(_))));
        assert!(has_list);
    }

    #[test]
    fn test_code_block_parsing() {
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_code_block = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::CodeBlock(_))));
        assert!(has_code_block);
    }

    #[test]
    fn test_link_parsing() {
        let markdown = "[Link text](https://example.com)";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_link = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Link(_, _, _))));
        assert!(has_link);
    }

    #[test]
    fn test_image_parsing() {
        let markdown = "![Alt text](image.png)";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_image = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Image(_, _, _))));
        assert!(has_image);
    }

    #[test]
    fn test_video_link_detection() {
        let markdown = "[video](video.mp4)";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        // Check for link that could be a video
        let has_video_link = events.iter().any(|e| {
            if let Event::Start(Tag::Link(_, url, _)) = e {
                url.ends_with(".mp4") || url.ends_with(".webm") || url.ends_with(".avi")
            } else {
                false
            }
        });
        assert!(has_video_link);
    }

    #[test]
    fn test_emphasis_parsing() {
        let markdown = "*italic* and **bold** text";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_emphasis = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Emphasis)));
        let has_strong = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Strong)));

        assert!(has_emphasis);
        assert!(has_strong);
    }

    #[test]
    fn test_blockquote_parsing() {
        let markdown = "> This is a quote\n> Second line";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_blockquote = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::BlockQuote)));
        assert!(has_blockquote);
    }

    #[test]
    fn test_table_parsing() {
        let markdown = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_table = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Table(_))));
        assert!(has_table);
    }

    #[test]
    fn test_inline_code_parsing() {
        let markdown = "This is `inline code` in text";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_inline_code = events.iter().any(|e| matches!(e, Event::Code(_)));
        assert!(has_inline_code);
    }

    #[test]
    fn test_horizontal_rule_parsing() {
        let markdown = "Text above\n\n---\n\nText below";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        let has_rule = events.iter().any(|e| matches!(e, Event::Rule));
        assert!(has_rule);
    }

    #[test]
    fn test_nested_lists() {
        let markdown = "- Item 1\n  - Nested 1\n  - Nested 2\n- Item 2";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        // Count list starts
        let list_count = events
            .iter()
            .filter(|e| matches!(e, Event::Start(Tag::List(_))))
            .count();
        assert!(list_count >= 2); // Main list and nested list
    }

    #[test]
    fn test_mixed_content() {
        let markdown = "# Title\n\nSome **bold** and *italic* text with `code`.\n\n```rust\nfn test() {}\n```\n\n[Link](url)";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();

        // Should contain various elements
        let has_heading = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Heading(_, _, _))));
        let has_strong = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Strong)));
        let has_emphasis = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Emphasis)));
        let has_code = events.iter().any(|e| matches!(e, Event::Code(_)));
        let has_code_block = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::CodeBlock(_))));
        let has_link = events
            .iter()
            .any(|e| matches!(e, Event::Start(Tag::Link(_, _, _))));

        assert!(has_heading);
        assert!(has_strong);
        assert!(has_emphasis);
        assert!(has_code);
        assert!(has_code_block);
        assert!(has_link);
    }
}
