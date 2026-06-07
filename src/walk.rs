//! Parallel, gitignore-aware discovery of source files.

use crate::lang::Lang;
use ignore::WalkBuilder;
use std::path::PathBuf;

/// A source file we intend to parse, paired with its detected language.
pub struct SourceFile {
    pub path: PathBuf,
    pub lang: Lang,
}

/// Walk `root` respecting .gitignore, returning all files whose extension
/// maps to a supported language. Hidden files and common vendor dirs are
/// skipped by the `ignore` crate's defaults plus our overrides.
pub fn discover(root: &str) -> Vec<SourceFile> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .parents(true);

    // Skip directories that are almost never what you want to navigate.
    for dir in ["node_modules", "target", "dist", "build", "vendor", ".venv"] {
        let _ = builder.add_ignore(dir);
    }

    let mut out = Vec::new();
    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.into_path();
        let path_str = path.to_string_lossy();
        if let Some(lang) = Lang::from_path(&path_str) {
            out.push(SourceFile { path, lang });
        }
    }
    out
}
