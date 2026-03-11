mod parser;
mod reporter;
mod scanner;
mod types;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use types::OutputFormat;

/// A CLI tool that scans codebases for TODO, FIXME, HACK, and other annotation comments.
#[derive(Parser)]
#[command(name = "todohawk", version, about)]
struct Cli {
    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,

    /// Filter by tag (can be repeated, e.g. --tag TODO --tag FIXME)
    #[arg(short, long)]
    tag: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut items = scanner::scan_directory(&cli.path)?;

    // Filter by tags if specified
    if !cli.tag.is_empty() {
        let tags: Vec<String> = cli.tag.iter().map(|t| t.to_uppercase()).collect();
        items.retain(|item| tags.contains(&item.tag.to_string()));
    }

    let output = reporter::report(&items, &cli.format)?;
    print!("{output}");

    Ok(())
}
