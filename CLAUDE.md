# Claude

## Build

```
cargo check
```

## Lint and format

- `cargo fmt --all` formats; `cargo clippy --workspace --all-targets -- -D warnings` lints. Run both before committing.
- A checked-in pre-commit hook (`.githooks/pre-commit`) runs the fmt check and clippy on every commit, so the same gate applies to humans and Claude. A commit touching `projects/utilities/sdfj-builder` also runs `deno check`, `deno lint`, `deno fmt --check`, and `deno test` there. A commit touching `projects/utilities/voxrender-bench-quest` also runs clippy on it for `aarch64-linux-android`.
- Enable the hook once after cloning with `npm run setup` (or `git config core.hooksPath .githooks`); see the README Development section.

## Style

- Rust edition 2024
- Consolidate imports into minimal nested `use` statements with no duplicate path prefixes (e.g., `use std::{fs, io::{ErrorKind, Write}, path::{Path, PathBuf}}` not separate `use std::fs; use std::io::Write;`)
- Code bodies name items through `use` imports, never fully-qualified paths
- Import types/traits/enums as leaf items, with aliases to avoid collisions (e.g., `Error as IOError`, `Result as StdResult`, `Error as StdError`)
- Import modules for free functions and keep the module prefix in calls (e.g., `use std::{env, fs, io, process};` then `fs::read()`, `env::temp_dir()`, `io::stdout()`, `process::exit(1)`)
- No preludes. A glob `use` appears only in a parent's `pub use module::*;` re-exports and in test modules. A crate's root re-exports its extension traits for callers to import as leaf items
- Prefer `#[derive(Default)]` over manual `impl Default` when all field defaults match the type's inherent default
- A blank line separates each top-level item in a file and each member of a struct, enum, trait, or `impl` (fields, variants, methods, associated types and consts); `use` lines and `mod` declarations stay grouped
- A blank line separates every arm of a `match` in which any arm spans more than one line; a match whose arms are all one line stays compact
- One public item per file (struct, trait, enum, or function), file named to match the item in snake_case; capability methods on a type from another file ride an extension trait, one trait per file
- A file holding only consts may hold several related ones (e.g., a material vocabulary's keys and defaults)
- A helper that only one file calls on another file's type lives in that file as a private free function taking the type as a parameter
  - When another file's tests also call the helper, it gets its own crate-internal file instead
  - A `pub(crate)` method that needs the type's private fields stays on the type, whatever its callers
- A type's inherent `impl`s go where their feature gate puts them:
  1. An `impl` under the type's gate lives in the type's file
  2. An `impl` under a narrower gate lives in `{type}_{module}_ext.rs`, named for the module holding the file, with the `cfg` on the `mod` line and never on a method
  3. A narrower gate that two or more files share gets its own submodule
  4. A type gets at most one ext file per module, so a second narrower gate for the type moves into a submodule
  5. When callers in several modules share a narrower-gated method, the type's file becomes a submodule holding `{type}.rs` plus one `{method}.rs` per gate, with each `cfg` on its `mod` line
- Doc comments (`///`) on all public items
- `#[arg]` attributes always start with `value_name` (e.g., `#[arg(value_name = "input-fbx")]`, `#[arg(value_name = "max-iterations", long)]`)

## Errors

- Bias toward errors over hiding them. A value that does not fit its
  destination is an error, never silently clamped, coerced, defaulted, or
  dropped.
- A defensive fallback that papers over a check made upstream is the same bug.
  `expect` what an earlier validation guarantees, so a gap fails with its reason
  instead of quietly substituting a default.

## Module structure

- A parent `mod.rs` or `lib.rs` declares each one-public-item file as a private `mod`
- `mod.rs` / `lib.rs` files group declarations under these section comments, in this order, and drop empty sections. voxj-voxcore's `lib.rs` shows the layout.
  1. `// Public API`: private `mod` declarations, then the `pub use module_name::*;` re-exports that flatten the public API
  2. `// Optional API`: feature-gated modules, each gate's `mod` followed by its re-export
  3. `// Internal API`: crate-internal modules
  4. `// Test support`: `#[cfg(test)]` modules
- Crate-internal items use `pub(crate) use module_name::*;`, and the items themselves are `pub`, since the private `mod` and the `pub(crate)` re-export already set their reach
- Subdirectories that consumers navigate are declared `pub mod` (e.g., `pub mod commands;`)
- Leaf files are always private modules whose public items the parent re-exports
- Prefer `use crate` over `use super`

## Feature gates

- Each library crate has a default `impl` feature that gates the concrete `DependenciesImpl` and any deps it needs (e.g., `glob`, `tyt-injection`)
- `#[cfg(feature = "impl")]` guards `mod dependencies_impl` and its `pub use` in `lib.rs`
- The parent `tyt` crate's `impl` feature transitively enables sub-crate `impl` features
- A `cfg` switch between alternative definitions stays in one file, so a stand-in for a gated-off item lives beside the code that picks it

## Architecture

- `tyt` is the top-level binary that ties sub-crates together via `clap` subcommands
- Every tyt crate depends non-optionally on `tyt-common`, which provides shared types (e.g., `ExecFailed`)
- Sub-crate `DependenciesImpl`s take shared helper free functions from `tyt-injection`, which each crate's `impl` feature pulls in as an optional dependency
- Each sub-crate (`tyt-fbx`, `tyt-material`) has a `Dependencies` trait for dependency injection and a feature-gated `DependenciesImpl`
- The `tyt` crate bridges sub-crate dependencies through associated types on its own `Dependencies` trait
