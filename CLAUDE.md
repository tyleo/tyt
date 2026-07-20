# Claude

## Build

```
cargo check
```

## Lint and format

- `cargo fmt --all` formats; `cargo clippy --workspace --all-targets -- -D warnings` lints. Run both before committing.
- A checked-in pre-commit hook (`.githooks/pre-commit`) runs the fmt check and clippy on every commit, so the same gate applies to humans and Claude.
- Enable the hook once after cloning with `npm run setup` (or `git config core.hooksPath .githooks`); see the README Development section.

## Style

- Rust edition 2024
- Consolidate imports into minimal nested `use` statements — no duplicate path prefixes (e.g., `use std::{fs, io::{ErrorKind, Write}, path::{Path, PathBuf}}` not separate `use std::fs; use std::io::Write;`)
- No fully-qualified paths in code bodies — import all referenced items in `use` blocks
- Import types/traits/enums as leaf items, with aliases to avoid collisions (e.g., `Error as IOError`, `Result as StdResult`, `Error as StdError`)
- Import modules for free functions and keep the module prefix in calls (e.g., `use std::{env, fs, io, process};` then `fs::read()`, `env::temp_dir()`, `io::stdout()`, `process::exit(1)`)
- Prefer `#[derive(Default)]` over manual `impl Default` when all field defaults match the type's inherent default
- One public item per file (struct, trait, enum, or function), file named to match the item in snake_case; capability methods on a type from another file ride an extension trait, one trait per file
- Impl-only files: a file adding a single method to another file's type is named for the method (e.g., `resolve_json.rs`); a constructor-family file keeps the extended type's name
- Doc comments (`///`) on all public items
- `#[arg]` attributes always start with `value_name` (e.g., `#[arg(value_name = "input-fbx")]`, `#[arg(value_name = "max-iterations", long)]`)

## Module structure

- One public item per file — the file is a private `mod` in its parent `mod.rs` or `lib.rs`
- `mod.rs` / `lib.rs` files have two sections: private `mod` declarations, then `pub use module_name::*;` re-exports to flatten the public API
- Crate-internal items use `pub(crate) use module_name::*;`
- Subdirectories that consumers navigate are declared `pub mod` (e.g., `pub mod commands;`)
- Leaf files are always private modules — their public items are re-exported by the parent
- Prefer `use crate` over `use super`

## Feature gates

- Each library crate has a default `impl` feature that gates the concrete `DependenciesImpl` and any deps it needs (e.g., `glob`, `tyt-injection`)
- `#[cfg(feature = "impl")]` guards `mod dependencies_impl` and its `pub use` in `lib.rs`
- The parent `tyt` crate's `impl` feature transitively enables sub-crate `impl` features

## Architecture

- `tyt` is the top-level binary that ties sub-crates together via `clap` subcommands
- `tyt-common` provides shared types (e.g., `ExecFailed`) used across all tyt crates — every crate depends on it non-optionally
- `tyt-injection` provides shared implementation helpers (free functions) used by sub-crate `DependenciesImpl`s — depended on optionally behind the `impl` feature
- Each sub-crate (`tyt-fbx`, `tyt-material`) has a `Dependencies` trait for dependency injection and a feature-gated `DependenciesImpl`
- The `tyt` crate bridges sub-crate dependencies through associated types on its own `Dependencies` trait
