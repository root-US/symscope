//! Symbol extraction: parse each source file with tree-sitter and collect
//! definition sites and call sites into a flat, queryable index.

use crate::lang::{parser_for, Lang};
use crate::walk::SourceFile;
use anyhow::Result;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tree_sitter::{Query, QueryCursor};

/// The kind of symbol occurrence we recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Def,
    Call,
}

/// A single occurrence of a named symbol in the codebase.
#[derive(Debug, Clone, Serialize)]
pub struct Symbol {
    pub name: String,
    pub kind: Kind,
    pub file: PathBuf,
    pub line: usize,  // 1-based
    pub col: usize,   // 1-based
    pub text: String, // the source line, trimmed of trailing whitespace
    /// For a call site, the name of the function/method that contains it
    /// (the caller). `None` for top-level calls and for definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclosing: Option<String>,
}

/// The full index over a tree of source files.
#[derive(Default)]
pub struct Index {
    pub symbols: Vec<Symbol>,
    pub files_parsed: usize,
}

impl Index {
    /// Build an index by parsing every discovered file in parallel.
    pub fn build(files: &[SourceFile]) -> Index {
        let collected: Mutex<Vec<Symbol>> = Mutex::new(Vec::new());
        let count: Mutex<usize> = Mutex::new(0);

        files.par_iter().for_each(|sf| {
            if let Ok(mut syms) = index_one(sf) {
                if !syms.is_empty() {
                    collected.lock().unwrap().append(&mut syms);
                }
                *count.lock().unwrap() += 1;
            }
        });

        Index {
            symbols: collected.into_inner().unwrap(),
            files_parsed: count.into_inner().unwrap(),
        }
    }

    /// All definition sites matching `name` exactly.
    pub fn definitions<'a>(&'a self, name: &str) -> Vec<&'a Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.kind == Kind::Def && s.name == name)
            .collect()
    }

    /// All call sites whose callee matches `name` exactly.
    pub fn callers<'a>(&'a self, name: &str) -> Vec<&'a Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.kind == Kind::Call && s.name == name)
            .collect()
    }

    /// All occurrences (defs + calls) of `name`.
    pub fn references<'a>(&'a self, name: &str) -> Vec<&'a Symbol> {
        self.symbols.iter().filter(|s| s.name == name).collect()
    }

    /// All caller -> callee edges in the codebase, derived from call sites
    /// that sit inside a known function. Each edge is `(caller, callee)`.
    pub fn edges(&self) -> Vec<(String, String)> {
        self.symbols
            .iter()
            .filter(|s| s.kind == Kind::Call)
            .filter_map(|s| s.enclosing.clone().map(|caller| (caller, s.name.clone())))
            .collect()
    }
}

/// Parse a single file and extract its definition and call symbols.
fn index_one(sf: &SourceFile) -> Result<Vec<Symbol>> {
    let src = fs::read_to_string(&sf.path)?;
    let mut parser = parser_for(sf.lang)?;
    let tree = match parser.parse(&src, None) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut out = Vec::new();
    let bytes = src.as_bytes();
    let lines: Vec<&str> = src.lines().collect();

    run_query(
        sf,
        sf.lang.def_query(),
        Kind::Def,
        &tree,
        bytes,
        &lines,
        &mut out,
    )?;
    run_query(
        sf,
        sf.lang.call_query(),
        Kind::Call,
        &tree,
        bytes,
        &lines,
        &mut out,
    )?;

    Ok(out)
}

fn run_query(
    sf: &SourceFile,
    query_src: &str,
    kind: Kind,
    tree: &tree_sitter::Tree,
    bytes: &[u8],
    lines: &[&str],
    out: &mut Vec<Symbol>,
) -> Result<()> {
    let query = Query::new(&sf.lang.grammar(), query_src)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), bytes);

    for m in matches {
        for cap in m.captures {
            let node = cap.node;
            let name = match node.utf8_text(bytes) {
                Ok(t) => t.to_string(),
                Err(_) => continue,
            };
            let start = node.start_position();
            let line_idx = start.row;
            let text = lines
                .get(line_idx)
                .map(|l| l.trim_end().to_string())
                .unwrap_or_default();
            // For call sites, find the enclosing function/method (the caller)
            // by walking up the syntax tree. Definitions don't need this.
            let enclosing = if kind == Kind::Call {
                enclosing_function(node, sf.lang, bytes)
            } else {
                None
            };
            out.push(Symbol {
                name,
                kind,
                file: sf.path.clone(),
                line: line_idx + 1,
                col: start.column + 1,
                text,
                enclosing,
            });
        }
    }
    Ok(())
}

/// Walk up from `node` to the nearest enclosing function/method definition
/// and return its name. Returns `None` if the call is at top level.
fn enclosing_function(node: tree_sitter::Node, lang: Lang, bytes: &[u8]) -> Option<String> {
    let mut cur = node.parent();
    while let Some(n) = cur {
        if lang.is_function_def_kind(n.kind()) {
            // The defining identifier is the child captured by the def query;
            // grab the node's "name" field if present, else the first
            // identifier-like child.
            if let Some(name_node) = n
                .child_by_field_name("name")
                .or_else(|| first_identifier_child(n))
            {
                if let Ok(t) = name_node.utf8_text(bytes) {
                    return Some(t.to_string());
                }
            }
        }
        cur = n.parent();
    }
    None
}

/// Best-effort: find the first child that looks like an identifier, used as a
/// fallback when a grammar doesn't expose a `name` field on its def node.
fn first_identifier_child(node: tree_sitter::Node) -> Option<tree_sitter::Node> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let k = child.kind();
        if k.contains("identifier") {
            return Some(child);
        }
    }
    None
}
