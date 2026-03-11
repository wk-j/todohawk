use serde::Serialize;
use std::fmt;
use std::path::PathBuf;

/// Supported annotation tags
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Tag {
    Todo,
    Fixme,
    Hack,
    Xxx,
    Note,
    Optimize,
    Bug,
    Warn,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tag::Todo => write!(f, "TODO"),
            Tag::Fixme => write!(f, "FIXME"),
            Tag::Hack => write!(f, "HACK"),
            Tag::Xxx => write!(f, "XXX"),
            Tag::Note => write!(f, "NOTE"),
            Tag::Optimize => write!(f, "OPTIMIZE"),
            Tag::Bug => write!(f, "BUG"),
            Tag::Warn => write!(f, "WARN"),
        }
    }
}

impl Tag {
    pub fn all() -> Vec<Tag> {
        vec![
            Tag::Todo,
            Tag::Fixme,
            Tag::Hack,
            Tag::Xxx,
            Tag::Note,
            Tag::Optimize,
            Tag::Bug,
            Tag::Warn,
        ]
    }
}

/// A single TODO-style comment found in the codebase
#[derive(Debug, Clone, Serialize)]
pub struct TodoItem {
    /// The annotation tag (TODO, FIXME, etc.)
    pub tag: Tag,
    /// The message content after the tag
    pub message: String,
    /// Optional author from TAG(author) syntax
    pub author: Option<String>,
    /// File path where the item was found
    pub file: PathBuf,
    /// Line number (1-based)
    pub line: usize,
    /// The full raw line content
    pub raw_line: String,
}

/// Output format for reporting
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Markdown,
    Summary,
}
