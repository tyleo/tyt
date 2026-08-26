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

let prefs: Prefs<MyPrefs> =
    load_prefs(&DependenciesImpl, &JsoncCodec, ".appconfig", "my-tool")?;

for (dir, layer) in prefs.application_order() {
    // Later layers override earlier ones.
}
```

## Layers

`load_prefs` reads the named file in every location and returns a `Prefs<T>`
holding three layers:

1. `user`: the user home directory
2. `git_root`: the git repository root, `None` outside a repository
3. `hierarchy`: the directories from the git root down to cwd that supplied
   prefs, furthest from cwd first

`application_order` yields `(dir, prefs)` pairs: the user layer first, then
the hierarchy down to cwd. A caller merging in that order lets the layer
nearest cwd win.

Narrower loaders read a subset: `load_user_prefs`, `load_git_prefs`, and
`load_hierarchy_prefs`. `load_application_prefs` returns the layers that
supplied prefs in application order.

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

Filesystem access rides the `Dependencies` trait. The built-in
`DependenciesImpl` reads real files, resolves the git root with
`git rev-parse`, and replaces files atomically through a sibling temp file.
No feature gates it, and it pulls in no crates. Callers can supply their own
implementation for tests or sandboxing.
