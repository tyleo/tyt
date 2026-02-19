# Claude

## Build

```
cargo check
```

## Style

- Rust edition 2024
- Consolidate imports into minimal nested `use` statements — no duplicate path prefixes (e.g., `use std::{fs, io::{ErrorKind, Write}, path::{Path, PathBuf}}` not separate `use std::fs; use std::io::Write;`)
- Prefer `#[derive(Default)]` over manual `impl Default` when all field defaults match the type's inherent default
- One public struct/trait/enum per file, file named to match the type in snake_case
- Doc comments (`///`) on all public items
- `#[arg]` attributes always start with `value_name` (e.g., `#[arg(value_name = "input-fbx")]`, `#[arg(value_name = "max-iterations", long)]`)

## Module structure

- One public struct/trait/enum per file — the file is a private `mod` in its parent `mod.rs` or `lib.rs`
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
