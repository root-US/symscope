//! symscope — fast, local code navigation.
//!
//! Subcommands:
//!   symscope def     <name>  Find where a symbol is defined.
//!   symscope refs    <name>  Find every reference (defs + calls) to a symbol.
//!   symscope callers <name>  Find every call site of a function/method.
//!   symscope stats           Summary of the codebase the index sees.

mod graph;
mod index;
mod lang;
mod output;
mod walk;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::tty::IsTty;
use index::Index;
use std::io::stdout;

#[derive(Parser)]
#[command(
    name = "symscope",
    version,
    about = "Fast, local code navigation powered by tree-sitter.",
    long_about = "symscope answers structural questions about a codebase \
                  instantly and locally: where a symbol is defined, every place \
                  it is referenced, and every site that calls a function. \
                  No index server, no LLM, no network."
)]
struct Cli {
    /// Root directory to search (defaults to the current directory).
    #[arg(short, long, default_value = ".", global = true)]
    path: String,

    /// Emit machine-readable JSON instead of human output.
    #[arg(long, global = true)]
    json: bool,

    /// Force-disable colored output.
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Find where SYMBOL is defined.
    Def { symbol: String },
    /// Find every reference to SYMBOL (definitions and calls).
    Refs { symbol: String },
    /// Find every call site of SYMBOL.
    Callers { symbol: String },
    /// Print a call graph rooted at SYMBOL.
    Graph {
        symbol: String,
        /// Traverse callers (who calls SYMBOL) instead of callees.
        #[arg(long)]
        callers: bool,
        /// Maximum traversal depth.
        #[arg(long, default_value_t = 3)]
        depth: usize,
        /// Output format.
        #[arg(long, value_enum, default_value_t = GraphFormat::Tree)]
        format: GraphFormat,
    },
    /// Print a summary of the indexed codebase.
    Stats,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum GraphFormat {
    Tree,
    Dot,
    Mermaid,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let files = walk::discover(&cli.path);
    let index = Index::build(&files);

    match &cli.command {
        Command::Def { symbol } => {
            let results = index.definitions(symbol);
            emit("definitions", &results, &cli);
        }
        Command::Refs { symbol } => {
            let results = index.references(symbol);
            emit("references", &results, &cli);
        }
        Command::Callers { symbol } => {
            let results = index.callers(symbol);
            emit("callers", &results, &cli);
        }
        Command::Graph {
            symbol,
            callers,
            depth,
            format,
        } => {
            let g = graph::Graph::from_edges(&index.edges());
            let dir = if *callers {
                graph::Direction::Callers
            } else {
                graph::Direction::Callees
            };
            let fmt = match format {
                GraphFormat::Tree => graph::Format::Tree,
                GraphFormat::Dot => graph::Format::Dot,
                GraphFormat::Mermaid => graph::Format::Mermaid,
            };
            let edges = g.reachable_edges(symbol, dir, *depth);
            if edges.is_empty() {
                eprintln!(
                    "no {} found for '{symbol}' (is it defined/called in this tree?)",
                    if *callers { "callers" } else { "callees" }
                );
            } else {
                print!("{}", graph::render(symbol, &edges, dir, fmt));
            }
        }
        Command::Stats => {
            let defs = index
                .symbols
                .iter()
                .filter(|s| matches!(s.kind, index::Kind::Def))
                .count();
            let calls = index.symbols.len() - defs;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "files_parsed": index.files_parsed,
                        "definitions": defs,
                        "calls": calls,
                    }))?
                );
            } else {
                println!("files parsed: {}", index.files_parsed);
                println!("definitions:  {defs}");
                println!("call sites:   {calls}");
            }
        }
    }
    Ok(())
}

fn emit(label: &str, results: &[&index::Symbol], cli: &Cli) {
    if cli.json {
        output::render_json(results);
    } else {
        let use_color = !cli.no_color && stdout().is_tty();
        output::render_human(label, results, use_color);
    }
}
