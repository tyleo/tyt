# ty-preferences

Layered preference loading from sectioned config files.

A config file is an object whose top-level keys are sections. Each tool names
its config file and section key, so unrelated tools share one file without
reading each other's preferences.

```jsonc
{
  // Each tool reads its own section.
  "my-tool": {
    "greeting": "hello"
  }
}
```

```rust
#[derive(Deserialize)]
struct MyPrefs {
    greeting: Option<String>,
}

let paths = resolve_prefs_paths()?;

let prefs: Prefs<MyPrefs> = load_prefs(
    &DependenciesImpl,
    &JsoncCodec,
    &paths,
    ".appconfig",
    "my-tool",
)?;

for (dir, layer) in prefs.application_order() {
    // Later layers override earlier ones.
}
```

## Paths

`resolve_prefs_paths` builds a `PrefsPaths`: the cwd, the root of the git
repository containing it, and the user home directory. `git_root` and
`user` can be `None`; `cwd` is always present. One `PrefsPaths` can serve
many loads.

`resolve_cwd`, `resolve_user_home_dir`, and `resolve_git_root_dir` resolve
one location each; `resolve_git_root_dir` runs `git rev-parse`. The
`_from_cwd` variants take the starting directory instead of reading the
current one. Tests can skip resolution and build a `PrefsPaths` literally.

## Layers

`load_prefs` reads the named file at every `PrefsPaths` location and returns
a `Prefs<T>` holding three layers:

1. `user`: the layer from the user home directory
2. `git_root`: the layer from the git root
3. `hierarchy`: the layers from the git root down to cwd, furthest from cwd
   first

Each layer is a `DirPrefs`: a directory paired with the prefs it supplied.
Locations that supplied no prefs are absent.

`application_order` yields `(dir, prefs)` pairs: the user layer first, then
the hierarchy down to cwd. A caller merging in that order lets the layer
nearest cwd win.

Narrower loaders read a subset. `load_prefs_from_dir` reads one directory.
`load_sources_prefs` reads an ordered list of `(directory, file name)`
sources. `load_hierarchy_prefs` walks from a git root down to a cwd.
`load_application_prefs` returns the layers that supplied prefs in
application order.

## Sections

`read_section` reads one section from the config file at an explicit path.
`write_section` writes one section back and preserves every other top-level
section. It creates the file when none exists.

## Codecs

Every loader and section function takes the codec as an argument, so each
call picks its dialect. A codec implements `DeserializePrefs<T>`,
`SerializePrefs<T>`, or both.

The default `jsonc-codec` feature provides `JsoncCodec`: JSON plus comments
and trailing commas. Writing edits the file's concrete syntax tree and
preserves the comments and formatting of every untouched section.

The `json-codec` feature provides `JsonCodec` for strict JSON. Writing
rebuilds the file pretty-printed.

## Dependencies

File access rides the `Dependencies` trait: `read_file` and `write_file`.
The built-in `DependenciesImpl` reads real files and replaces them
atomically through a sibling temp file. No feature gates it, and it pulls
in no crates. Callers can supply their own implementation for tests or
sandboxing.
