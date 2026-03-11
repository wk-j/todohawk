use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::parser;
use crate::types::TodoItem;

/// Recursively scan a directory for TODO-style comments.
///
/// Walks the directory tree, skips hidden directories and common non-text
/// directories (target, node_modules, .git), reads each file line by line,
/// and delegates to the parser to extract TODO items.
pub fn scan_directory(path: &Path) -> Result<Vec<TodoItem>> {
    let mut items = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored(e))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip entries we can't read
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();

        // Skip binary / non-text files by checking extension
        if !is_scannable(file_path) {
            continue;
        }

        match scan_file(file_path) {
            Ok(file_items) => items.extend(file_items),
            Err(_) => continue, // skip files we can't read (binary, permission, etc.)
        }
    }

    Ok(items)
}

/// Scan a single file for TODO items.
fn scan_file(path: &Path) -> Result<Vec<TodoItem>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let mut items = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        if let Some(item) = parser::parse_line(line, path, line_number + 1) {
            items.push(item);
        }
    }

    Ok(items)
}

/// Returns true if a walkdir entry should be skipped.
fn is_ignored(entry: &walkdir::DirEntry) -> bool {
    // Never skip the root directory (depth 0)
    if entry.depth() == 0 {
        return false;
    }

    let name = entry.file_name().to_string_lossy();

    // Skip hidden files/directories
    if name.starts_with('.') {
        return true;
    }

    // Skip common non-source directories
    let skip_dirs = [
        "target",
        "node_modules",
        "dist",
        "build",
        "vendor",
        "__pycache__",
    ];
    if entry.file_type().is_dir() && skip_dirs.contains(&name.as_ref()) {
        return true;
    }

    false
}

/// Check if a file is likely a text source file we should scan.
fn is_scannable(path: &Path) -> bool {
    // If there's no extension, still try to scan (could be Makefile, Dockerfile, etc.)
    let Some(ext) = path.extension() else {
        return true;
    };
    let ext = ext.to_string_lossy().to_lowercase();

    // Skip known binary extensions
    let binary_exts = [
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp", // images
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", // documents
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar", // archives
        "exe", "dll", "so", "dylib", "o", "a", "lib", // binaries
        "wasm", "class", "pyc", "pyo", // compiled
        "ttf", "otf", "woff", "woff2", "eot", // fonts
        "mp3", "mp4", "avi", "mov", "wav", "flac", // media
        "lock", // lock files (often huge)
    ];

    !binary_exts.contains(&ext.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Create a file with a TODO comment
        let src = dir.path().join("main.rs");
        fs::write(&src, "fn main() {\n    // TODO: implement this\n}\n").unwrap();

        // Create a file in a subdirectory
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("lib.rs"), "// FIXME: broken\nfn foo() {}\n").unwrap();

        // Create a hidden directory that should be skipped
        let hidden = dir.path().join(".hidden");
        fs::create_dir(&hidden).unwrap();
        fs::write(hidden.join("secret.rs"), "// TODO: should not appear\n").unwrap();

        // Create a binary file that should be skipped
        fs::write(dir.path().join("image.png"), &[0x89, 0x50, 0x4e, 0x47]).unwrap();

        dir
    }

    #[test]
    fn test_is_ignored_hidden() {
        let dir = TempDir::new().unwrap();
        let hidden = dir.path().join(".git");
        fs::create_dir(&hidden).unwrap();

        for entry in WalkDir::new(dir.path()) {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy();
            if name == ".git" {
                assert!(is_ignored(&entry));
            }
        }
    }

    #[test]
    fn test_is_ignored_target() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("target");
        fs::create_dir(&target).unwrap();

        for entry in WalkDir::new(dir.path()) {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy();
            if name == "target" {
                assert!(is_ignored(&entry));
            }
        }
    }

    #[test]
    fn test_is_scannable() {
        assert!(is_scannable(Path::new("foo.rs")));
        assert!(is_scannable(Path::new("bar.py")));
        assert!(is_scannable(Path::new("Makefile")));
        assert!(!is_scannable(Path::new("image.png")));
        assert!(!is_scannable(Path::new("archive.zip")));
        assert!(!is_scannable(Path::new("Cargo.lock")));
    }

    #[test]
    fn test_scan_directory_skips_hidden() {
        let dir = create_test_dir();
        let items = scan_directory(dir.path()).unwrap();

        // Should not include items from .hidden directory
        for item in &items {
            assert!(
                !item.file.to_string_lossy().contains(".hidden"),
                "Should not scan hidden directories"
            );
        }
    }

    #[test]
    fn test_scan_directory_finds_items() {
        let dir = create_test_dir();
        let items = scan_directory(dir.path()).unwrap();

        // Should find items in main.rs and sub/lib.rs (but not .hidden)
        assert!(
            items.len() >= 2,
            "Expected at least 2 TODO items, found {}",
            items.len()
        );
    }
}
