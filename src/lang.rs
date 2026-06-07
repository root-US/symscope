//! Language detection and the tree-sitter queries used to extract symbols.
//!
//! Each supported language maps a file extension to (a) its tree-sitter
//! grammar and (b) a small set of S-expression queries that capture the
//! nodes we care about: definitions, calls, and identifier references.

use anyhow::{anyhow, Result};
use tree_sitter::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
}

impl Lang {
    /// Detect a language from a file path's extension.
    pub fn from_path(path: &str) -> Option<Lang> {
        let ext = path.rsplit('.').next()?;
        Some(match ext {
            "rs" => Lang::Rust,
            "py" | "pyi" => Lang::Python,
            "js" | "mjs" | "cjs" | "jsx" => Lang::JavaScript,
            "ts" | "tsx" => Lang::TypeScript,
            "go" => Lang::Go,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Go => "go",
        }
    }

    /// The tree-sitter grammar for this language.
    pub fn grammar(self) -> Language {
        match self {
            Lang::Rust => tree_sitter_rust::language(),
            Lang::Python => tree_sitter_python::language(),
            Lang::JavaScript => tree_sitter_javascript::language(),
            Lang::TypeScript => tree_sitter_typescript::language_typescript(),
            Lang::Go => tree_sitter_go::language(),
        }
    }

    /// Node kinds that represent a function/method definition for this
    /// language. Used to find the enclosing caller of a call site.
    pub fn is_function_def_kind(self, kind: &str) -> bool {
        match self {
            Lang::Rust => matches!(kind, "function_item"),
            Lang::Python => matches!(kind, "function_definition"),
            Lang::JavaScript | Lang::TypeScript => matches!(
                kind,
                "function_declaration" | "method_definition" | "function" | "arrow_function"
            ),
            Lang::Go => matches!(kind, "function_declaration" | "method_declaration"),
        }
    }

    /// A query that captures definition sites as `@def.name`.
    /// We intentionally keep these simple and robust across grammar versions.
    pub fn def_query(self) -> &'static str {
        match self {
            Lang::Rust => {
                r#"
                (function_item name: (identifier) @def.name)
                (struct_item name: (type_identifier) @def.name)
                (enum_item name: (type_identifier) @def.name)
                (trait_item name: (type_identifier) @def.name)
                (const_item name: (identifier) @def.name)
                (static_item name: (identifier) @def.name)
                (mod_item name: (identifier) @def.name)
                "#
            }
            Lang::Python => {
                r#"
                (function_definition name: (identifier) @def.name)
                (class_definition name: (identifier) @def.name)
                "#
            }
            Lang::JavaScript | Lang::TypeScript => {
                r#"
                (function_declaration name: (identifier) @def.name)
                (class_declaration name: (identifier) @def.name)
                (method_definition name: (property_identifier) @def.name)
                (variable_declarator name: (identifier) value: (arrow_function))
                "#
            }
            Lang::Go => {
                r#"
                (function_declaration name: (identifier) @def.name)
                (method_declaration name: (field_identifier) @def.name)
                (type_declaration (type_spec name: (type_identifier) @def.name))
                "#
            }
        }
    }

    /// A query that captures call sites' callee identifiers as `@call.name`.
    pub fn call_query(self) -> &'static str {
        match self {
            Lang::Rust => {
                r#"
                (call_expression function: (identifier) @call.name)
                (call_expression function: (field_expression field: (field_identifier) @call.name))
                (call_expression function: (scoped_identifier name: (identifier) @call.name))
                "#
            }
            Lang::Python => {
                r#"
                (call function: (identifier) @call.name)
                (call function: (attribute attribute: (identifier) @call.name))
                "#
            }
            Lang::JavaScript | Lang::TypeScript => {
                r#"
                (call_expression function: (identifier) @call.name)
                (call_expression function: (member_expression property: (property_identifier) @call.name))
                "#
            }
            Lang::Go => {
                r#"
                (call_expression function: (identifier) @call.name)
                (call_expression function: (selector_expression field: (field_identifier) @call.name))
                "#
            }
        }
    }
}

/// Build a tree-sitter parser configured for the given language.
pub fn parser_for(lang: Lang) -> Result<tree_sitter::Parser> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.grammar())
        .map_err(|e| anyhow!("failed to load grammar for {}: {e}", lang.name()))?;
    Ok(parser)
}
