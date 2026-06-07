//! Rendering query results either as colored human output or JSON.

use crate::index::Symbol;
use crossterm::style::Stylize;
use serde_json::json;

pub fn render_human(label: &str, results: &[&Symbol], use_color: bool) {
    if results.is_empty() {
        eprintln!("no {label} found");
        return;
    }
    for s in results {
        let loc = format!("{}:{}:{}", s.file.display(), s.line, s.col);
        if use_color {
            println!("{}  {}", loc.clone().cyan(), s.text.clone().dim());
        } else {
            println!("{}  {}", loc, s.text);
        }
    }
    if use_color {
        eprintln!("{}", format!("{} match(es)", results.len()).green());
    } else {
        eprintln!("{} match(es)", results.len());
    }
}

pub fn render_json(results: &[&Symbol]) {
    let arr: Vec<_> = results
        .iter()
        .map(|s| {
            json!({
                "name": s.name,
                "kind": s.kind,
                "file": s.file.to_string_lossy(),
                "line": s.line,
                "col": s.col,
                "text": s.text,
            })
        })
        .collect();
    // Pretty-print so piping to `jq` or reading by eye both work.
    match serde_json::to_string_pretty(&arr) {
        Ok(out) => println!("{out}"),
        Err(e) => eprintln!("failed to serialize: {e}"),
    }
}
