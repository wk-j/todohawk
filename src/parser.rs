use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use crate::types::{Tag, TodoItem};

/// Regex that matches TODO-style annotations in common comment styles.
///
/// Captures:
///   1: tag name (TODO, FIXME, etc.)
///   2: optional author (from `TAG(author)` syntax), without parens
///   3: message text after the tag (and optional colon/whitespace)
static TODO_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?://|#|/\*|<!--)\s*(TODO|FIXME|HACK|XXX|NOTE|OPTIMIZE|BUG|WARN)\s*(?:\(([^)]*)\))?\s*:?\s*(.*?)(?:\s*-->|\s*\*/)?$"
    )
    .expect("invalid TODO regex")
});

/// Parse a tag string into a `Tag` enum variant.
fn parse_tag(s: &str) -> Option<Tag> {
    match s {
        "TODO" => Some(Tag::Todo),
        "FIXME" => Some(Tag::Fixme),
        "HACK" => Some(Tag::Hack),
        "XXX" => Some(Tag::Xxx),
        "NOTE" => Some(Tag::Note),
        "OPTIMIZE" => Some(Tag::Optimize),
        "BUG" => Some(Tag::Bug),
        "WARN" => Some(Tag::Warn),
        _ => None,
    }
}

/// Parse a single line of source code for a TODO-style annotation.
///
/// Returns `Some(TodoItem)` if the line contains a recognized annotation,
/// or `None` if no match is found.
pub fn parse_line(line: &str, file: &Path, line_number: usize) -> Option<TodoItem> {
    let caps = TODO_RE.captures(line)?;

    let tag = parse_tag(caps.get(1)?.as_str())?;
    let author = caps.get(2).map(|m| m.as_str().trim().to_string()).filter(|s| !s.is_empty());
    let message = caps.get(3).map(|m| m.as_str().trim().to_string()).unwrap_or_default();

    Some(TodoItem {
        tag,
        message,
        author,
        file: file.to_path_buf(),
        line: line_number,
        raw_line: line.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn p(line: &str) -> Option<TodoItem> {
        parse_line(line, Path::new("test.rs"), 1)
    }

    // --- Basic comment styles ---

    #[test]
    fn double_slash_todo() {
        let item = p("// TODO: fix this").unwrap();
        assert_eq!(item.tag, Tag::Todo);
        assert_eq!(item.message, "fix this");
        assert!(item.author.is_none());
    }

    #[test]
    fn hash_fixme() {
        let item = p("# FIXME: broken logic").unwrap();
        assert_eq!(item.tag, Tag::Fixme);
        assert_eq!(item.message, "broken logic");
    }

    #[test]
    fn block_comment_hack() {
        let item = p("/* HACK: workaround for bug */").unwrap();
        assert_eq!(item.tag, Tag::Hack);
        assert_eq!(item.message, "workaround for bug");
    }

    #[test]
    fn html_comment_note() {
        let item = p("<!-- NOTE: deprecated section -->").unwrap();
        assert_eq!(item.tag, Tag::Note);
        assert_eq!(item.message, "deprecated section");
    }

    // --- Author syntax ---

    #[test]
    fn todo_with_author() {
        let item = p("// TODO(alice): refactor this").unwrap();
        assert_eq!(item.tag, Tag::Todo);
        assert_eq!(item.author, Some("alice".to_string()));
        assert_eq!(item.message, "refactor this");
    }

    #[test]
    fn fixme_with_author_hash() {
        let item = p("# FIXME(bob): memory leak").unwrap();
        assert_eq!(item.tag, Tag::Fixme);
        assert_eq!(item.author, Some("bob".to_string()));
        assert_eq!(item.message, "memory leak");
    }

    // --- All tag types ---

    #[test]
    fn all_tags_recognized() {
        let tags = [
            ("TODO", Tag::Todo),
            ("FIXME", Tag::Fixme),
            ("HACK", Tag::Hack),
            ("XXX", Tag::Xxx),
            ("NOTE", Tag::Note),
            ("OPTIMIZE", Tag::Optimize),
            ("BUG", Tag::Bug),
            ("WARN", Tag::Warn),
        ];
        for (name, expected) in tags {
            let line = format!("// {}: test message", name);
            let item = p(&line).unwrap_or_else(|| panic!("should match tag {}", name));
            assert_eq!(item.tag, expected);
            assert_eq!(item.message, "test message");
        }
    }

    // --- Edge cases ---

    #[test]
    fn no_match_returns_none() {
        assert!(p("let x = 42;").is_none());
        assert!(p("// just a comment").is_none());
        assert!(p("").is_none());
    }

    #[test]
    fn tag_without_colon() {
        let item = p("// TODO fix without colon").unwrap();
        assert_eq!(item.tag, Tag::Todo);
        assert_eq!(item.message, "fix without colon");
    }

    #[test]
    fn leading_whitespace() {
        let item = p("    // TODO: indented").unwrap();
        assert_eq!(item.tag, Tag::Todo);
        assert_eq!(item.message, "indented");
    }

    #[test]
    fn empty_message() {
        let item = p("// TODO:").unwrap();
        assert_eq!(item.tag, Tag::Todo);
        assert_eq!(item.message, "");
    }

    #[test]
    fn empty_author_parens_treated_as_no_author() {
        let item = p("// TODO(): message").unwrap();
        assert_eq!(item.tag, Tag::Todo);
        assert!(item.author.is_none());
        assert_eq!(item.message, "message");
    }

    #[test]
    fn file_and_line_are_recorded() {
        let file = PathBuf::from("src/main.rs");
        let item = parse_line("// TODO: check", &file, 42).unwrap();
        assert_eq!(item.file, file);
        assert_eq!(item.line, 42);
    }

    #[test]
    fn raw_line_preserved() {
        let line = "  // TODO(alice): do stuff";
        let item = p(line).unwrap();
        assert_eq!(item.raw_line, line);
    }

    #[test]
    fn block_comment_without_closing() {
        // A block comment opening without closing on the same line
        let item = p("/* BUG: multiline start").unwrap();
        assert_eq!(item.tag, Tag::Bug);
        assert_eq!(item.message, "multiline start");
    }

    #[test]
    fn lowercase_tag_not_matched() {
        assert!(p("// todo: lowercase").is_none());
    }
}
