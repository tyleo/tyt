# Implementation decisions

_Part of the [Vxl Command-Line Reference](../README.md)._

Code-level decisions made while building these commands, recorded as they land.
Command-design rationale lives in [design notes](design-notes.md); this log is
for implementation choices a reviewer of the Rust would want explained.

## MeshFormat

The doc comment names the planned `obj` and `gltf` variants even though only
`fbx` is implemented, because the checklist scopes this to fbx first and the
unbuilt variants are a stated plan rather than a hedge.

No `extension()` method yet. `Format` carries none, and the only caller is the
defaulted output path, which lands with the `mesh` command.
