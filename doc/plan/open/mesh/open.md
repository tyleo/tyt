# Open

_Part of the [mesh plan](README.md): the calls still open and what
settles them, with the doc TODOs at the end. The wider plan's
questions live in the
[implementation notes](implementation.md#open-questions)._

## Per-primitive normals

Every primitive writes `NORMAL` beside `POSITION`. glTF leaves the
attribute optional, a renderer deriving flat normals from the
triangles, and a data mesh may not want the bytes. Does `NORMAL`
toggle per primitive, and what spells the toggle?

## Vector functions

The set has no vector magnitude: no `length(e)` or its squared
twin, and no distance between two colors, the closeness test a
palette split wants. Which vector functions join the set, and under
what names?

## Enums

`alphaMode` takes `OPAQUE`, `MASK`, or `BLEND`, literals the value
language cannot spell, and reading them as bare tokens bolted an
enum surface onto the slot flag, so the design writes no enum
property today. What is the enum design, in the language or beside
it?

## TODOs

Documentation work rather than design calls.

1. A TypeScript schema for the whole
   [profile language](profile-language.md), the config shape in one
   block.
2. More worked examples: command lines, profiles, and the glTF they
   produce. A glTF snippet spells every field from the root down to
   the leaf it shows, large leaves collapsed to `// ...`.
