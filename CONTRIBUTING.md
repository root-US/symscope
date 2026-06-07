# Contributing to codenav

Thanks for your interest! Contributions of all kinds are welcome.

## Adding a new language

This is the highest-value contribution and is usually small:

1. Add the grammar crate to `Cargo.toml`, e.g. `tree-sitter-java = "0.21"`.
2. In `src/lang.rs`:
   - add a variant to `enum Lang`,
   - map its file extension(s) in `Lang::from_path`,
   - return its grammar in `Lang::grammar`,
   - write `def_query` and `call_query` S-expressions for it.
3. Add a fixture under `tests/fixtures/` and assertions in `tests/cli.rs`.

Use the [tree-sitter playground](https://tree-sitter.github.io/tree-sitter/playground)
to develop and verify your queries.

## Development

```bash
cargo build
cargo test
cargo fmt --all
cargo clippy --all-targets -- -D warnings
```

## Pull requests

- Keep PRs focused.
- Add or update tests for behavior changes.
- CI must pass (build + test on Linux/macOS/Windows, fmt, clippy).

By contributing you agree your work is dual-licensed under MIT OR Apache-2.0.
