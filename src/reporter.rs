use std::collections::BTreeMap;

use colored::Colorize;

use crate::types::{OutputFormat, Tag, TodoItem};

/// Format and report TODO items in the specified output format.
pub fn report(items: &[TodoItem], format: &OutputFormat) -> anyhow::Result<String> {
    match format {
        OutputFormat::Table => Ok(format_table(items)),
        OutputFormat::Json => format_json(items),
        OutputFormat::Markdown => Ok(format_markdown(items)),
        OutputFormat::Summary => Ok(format_summary(items)),
    }
}

fn colorize_tag(tag: &Tag) -> String {
    let label = tag.to_string();
    match tag {
        Tag::Fixme | Tag::Bug => label.red().bold().to_string(),
        Tag::Todo => label.yellow().bold().to_string(),
        Tag::Hack => label.magenta().bold().to_string(),
        Tag::Xxx => label.red().to_string(),
        Tag::Note => label.cyan().to_string(),
        Tag::Optimize => label.green().to_string(),
        Tag::Warn => label.truecolor(255, 165, 0).bold().to_string(),
    }
}

fn format_table(items: &[TodoItem]) -> String {
    if items.is_empty() {
        return "No TODO items found.".to_string();
    }

    let mut lines = Vec::new();

    // Header
    lines.push(format!(
        "{:<40} {:<6} {:<10} {:<12} {}",
        "File".bold().underline(),
        "Line".bold().underline(),
        "Tag".bold().underline(),
        "Author".bold().underline(),
        "Message".bold().underline(),
    ));

    for item in items {
        let file_str = item.file.display().to_string();
        let file_display = if file_str.len() > 38 {
            format!("..{}", &file_str[file_str.len() - 38..])
        } else {
            file_str
        };
        let author = item.author.as_deref().unwrap_or("-");
        lines.push(format!(
            "{:<40} {:<6} {:<10} {:<12} {}",
            file_display,
            item.line,
            colorize_tag(&item.tag),
            author,
            item.message,
        ));
    }

    lines.join("\n")
}

fn format_json(items: &[TodoItem]) -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(items)?)
}

fn format_markdown(items: &[TodoItem]) -> String {
    if items.is_empty() {
        return "No TODO items found.".to_string();
    }

    let mut grouped: BTreeMap<String, Vec<&TodoItem>> = BTreeMap::new();
    for item in items {
        let key = item.file.display().to_string();
        grouped.entry(key).or_default().push(item);
    }

    let mut lines = Vec::new();
    lines.push("# TODO Items".to_string());
    lines.push(String::new());

    for (file, file_items) in &grouped {
        lines.push(format!("## `{file}`"));
        lines.push(String::new());
        for item in file_items {
            let author_part = match &item.author {
                Some(a) => format!(" ({a})"),
                None => String::new(),
            };
            lines.push(format!(
                "- **{}**{} (line {}): {}",
                item.tag, author_part, item.line, item.message
            ));
        }
        lines.push(String::new());
    }

    // Remove trailing blank line
    if lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

fn format_summary(items: &[TodoItem]) -> String {
    if items.is_empty() {
        return "No TODO items found.".to_string();
    }

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in items {
        *counts.entry(item.tag.to_string()).or_default() += 1;
    }

    let total: usize = counts.values().sum();
    let parts: Vec<String> = counts
        .iter()
        .map(|(tag, count)| format!("{tag}: {count}"))
        .collect();

    format!("{}\nTotal: {total}", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_items() -> Vec<TodoItem> {
        vec![
            TodoItem {
                tag: Tag::Todo,
                message: "Refactor this function".to_string(),
                author: Some("alice".to_string()),
                file: PathBuf::from("src/main.rs"),
                line: 10,
                raw_line: "// TODO(alice): Refactor this function".to_string(),
            },
            TodoItem {
                tag: Tag::Fixme,
                message: "Handle error case".to_string(),
                author: None,
                file: PathBuf::from("src/main.rs"),
                line: 25,
                raw_line: "// FIXME: Handle error case".to_string(),
            },
            TodoItem {
                tag: Tag::Hack,
                message: "Temporary workaround".to_string(),
                author: Some("bob".to_string()),
                file: PathBuf::from("src/lib.rs"),
                line: 42,
                raw_line: "// HACK(bob): Temporary workaround".to_string(),
            },
        ]
    }

    #[test]
    fn test_table_format_contains_items() {
        let items = sample_items();
        let output = report(&items, &OutputFormat::Table).unwrap();
        assert!(output.contains("Refactor this function"));
        assert!(output.contains("Handle error case"));
        assert!(output.contains("Temporary workaround"));
        assert!(output.contains("alice"));
        assert!(output.contains("bob"));
    }

    #[test]
    fn test_table_empty() {
        let output = report(&[], &OutputFormat::Table).unwrap();
        assert_eq!(output, "No TODO items found.");
    }

    #[test]
    fn test_json_format() {
        let items = sample_items();
        let output = report(&items, &OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let arr = parsed.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["tag"], "TODO");
        assert_eq!(arr[0]["message"], "Refactor this function");
        assert_eq!(arr[0]["author"], "alice");
        assert_eq!(arr[0]["line"], 10);
        assert_eq!(arr[1]["tag"], "FIXME");
        assert!(arr[1]["author"].is_null());
        assert_eq!(arr[2]["tag"], "HACK");
    }

    #[test]
    fn test_json_empty() {
        let output = report(&[], &OutputFormat::Json).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_markdown_format() {
        let items = sample_items();
        let output = report(&items, &OutputFormat::Markdown).unwrap();
        assert!(output.starts_with("# TODO Items"));
        assert!(output.contains("## `src/main.rs`"));
        assert!(output.contains("## `src/lib.rs`"));
        assert!(output.contains("- **TODO** (alice) (line 10): Refactor this function"));
        assert!(output.contains("- **FIXME** (line 25): Handle error case"));
        assert!(output.contains("- **HACK** (bob) (line 42): Temporary workaround"));
    }

    #[test]
    fn test_markdown_empty() {
        let output = report(&[], &OutputFormat::Markdown).unwrap();
        assert_eq!(output, "No TODO items found.");
    }

    #[test]
    fn test_summary_format() {
        let items = sample_items();
        let output = report(&items, &OutputFormat::Summary).unwrap();
        assert!(output.contains("FIXME: 1"));
        assert!(output.contains("HACK: 1"));
        assert!(output.contains("TODO: 1"));
        assert!(output.contains("Total: 3"));
    }

    #[test]
    fn test_summary_empty() {
        let output = report(&[], &OutputFormat::Summary).unwrap();
        assert_eq!(output, "No TODO items found.");
    }

    #[test]
    fn test_summary_multiple_same_tag() {
        let items = vec![
            TodoItem {
                tag: Tag::Todo,
                message: "first".to_string(),
                author: None,
                file: PathBuf::from("a.rs"),
                line: 1,
                raw_line: "// TODO: first".to_string(),
            },
            TodoItem {
                tag: Tag::Todo,
                message: "second".to_string(),
                author: None,
                file: PathBuf::from("b.rs"),
                line: 2,
                raw_line: "// TODO: second".to_string(),
            },
        ];
        let output = report(&items, &OutputFormat::Summary).unwrap();
        assert!(output.contains("TODO: 2"));
        assert!(output.contains("Total: 2"));
    }
}
