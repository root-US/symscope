# codenav (`cn`)

**Fast, local code navigation — find definitions, references, and callers across your whole repo in milliseconds. No index server. No LLM. No network.**

[![CI](https://github.com/root-US/codenav/actions/workflows/ci.yml/badge.svg)](https://github.com/root-US/codenav/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/codenav.svg)](https://crates.io/crates/codenav)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`grep` finds text. `codenav` understands code. It parses your repository with
[tree-sitter](https://tree-sitter.github.io/) and answers the questions you
actually ask while reading code:

- **Where is this defined?** &nbsp;`cn def handleRequest`
- **Where is it used?** &nbsp;`cn refs handleRequest`
- **Who calls this function?** &nbsp;`cn callers handleRequest`
- **What's the call graph?** &nbsp;`cn graph handleRequest`

It walks your tree in parallel, respects `.gitignore`, and runs entirely on your
machine.

---

## Quickstart (30 seconds)

```bash
# Install (Rust 1.74+)
cargo install codenav

# From the root of any repo:
cn def parse_config        # where is parse_config defined?
cn callers parse_config    # every place that calls it
cn refs   parse_config     # defs + calls together
cn graph  parse_config     # call graph rooted at parse_config
cn stats                   # what does cn see in this repo?
```

Pipe structured output anywhere:

```bash
cn callers parse_config --json | jq '.[].file' | sort -u
```

## Call graphs

`cn graph` traces what a function calls (or, with `--callers`, who calls it),
to any depth, and renders it three ways:

```bash
cn graph parse_config                      # indented tree (default)
cn graph parse_config --callers            # invert: who reaches this function
cn graph parse_config --depth 5            # go deeper
cn graph parse_config --format mermaid     # paste into Markdown / GitHub
cn graph parse_config --format dot | dot -Tsvg > graph.svg
```

Tree output looks like:

```
parse_config
├── read_file
└── validate
    └── normalize
```

Cycles are detected and marked rather than looping forever.

---

## Why not just grep / ripgrep?

`ripgrep` is wonderful and `codenav` is not trying to replace it. The difference
is **structure**:

| Question | `grep`/`rg` | `codenav` |
|---|---|---|
| Find the text "parse" | ✅ great | not its job |
| Find where `parse()` is **defined** | ✗ matches comments, strings, calls | ✅ definition sites only |
| Find who **calls** `parse()` | ✗ can't tell a call from a mention | ✅ call sites only |

Because it parses real syntax trees, `cn` ignores matches inside comments and
strings, and it distinguishes a function *definition* from a *call* from a
*mention*.

## Why not an LSP or my IDE?

Language servers are powerful but heavy: per-language setup, a running daemon,
editor integration. `codenav` is a single static binary you run in any repo,
in any language it supports, with zero configuration — ideal for CI, scripts,
code review, and quick spelunking in unfamiliar code.

---

## Supported languages

Rust · Python · JavaScript · TypeScript · Go

More are easy to add — each language is a grammar plus a few tree-sitter
queries in [`src/lang.rs`](src/lang.rs). PRs welcome.

---

## Install

```bash
# From crates.io
cargo install codenav

# From source
git clone https://github.com/root-US/codenav
cd codenav
cargo install --path .
```

The binary is named `cn`.

---

## Usage

```
cn <COMMAND> [SYMBOL] [OPTIONS]

Commands:
  def      Find where SYMBOL is defined
  refs     Find every reference to SYMBOL (definitions and calls)
  callers  Find every call site of SYMBOL
  graph    Print a call graph rooted at SYMBOL
  stats    Print a summary of the indexed codebase

Options:
  -p, --path <PATH>   Root directory to search [default: .]
      --json          Emit machine-readable JSON (def/refs/callers)
      --no-color      Disable colored output
  -h, --help          Print help
  -V, --version       Print version

Graph options:
      --callers           Trace callers of SYMBOL instead of callees
      --depth <N>         Maximum traversal depth [default: 3]
      --format <FORMAT>   tree | dot | mermaid [default: tree]
```

---

## Roadmap

`codenav` is built as a fast Rust core with an ecosystem around it:

- [x] **v0.1** — Rust core: `def`, `refs`, `callers`, `graph`, `stats` across 5 languages
- [ ] **v0.2** — Python bindings via [PyO3](https://pyo3.rs/) (`pip install codenav`)
- [ ] **v0.3** — VS Code extension (inline callers, jump-to-def fallback)
- [ ] **v0.4** — Structural diff between two git revisions
- [ ] more languages (Java, C/C++, Ruby, C#)

---

## How it works

1. **Discover** — a parallel, `.gitignore`-aware walk (via the `ignore` crate
   that powers ripgrep) collects every source file in a supported language.
2. **Parse** — each file is parsed with its tree-sitter grammar, across all
   cores (via `rayon`).
3. **Extract** — a small set of S-expression queries per language captures
   definition sites and call sites into a flat index.
4. **Query** — `def`/`refs`/`callers` filter that index by name and kind.

There is no persisted index and no daemon: it is fast enough to rebuild from
scratch on every invocation for typical repositories.

---

## Contributing

Contributions are very welcome — especially new language support, which is
mostly a matter of adding a grammar dependency and its queries. See
[`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
