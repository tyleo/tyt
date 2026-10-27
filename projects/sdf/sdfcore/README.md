# sdfcore

The in-memory signed distance field model at the bottom of the SDF stack. Each
SDF format has a `-sdfcore` bridge crate that reads its documents into an
`SdfMain` and writes one back out.

## Architecture

`SdfState` holds a model's tables: 3D shapes, 2D shapes, materials, `shades`
calls, patterns, steps, and the parts' objects and nodes. Each table is an
`IdVec` keyed by a branded id. An entry references another entry by that
entry's id. `root_node_ids` lists the nodes at the top of the hierarchy.

Each table's entry type records one call per variant. `SdfShape3d::Box` holds
the corners and the optional rounding of a `box` call. A field left `None` takes
the call's default. A step holds an `SdfStepMaterial`, which picks a material or
a pattern.

## Validation

`SdfMain::new` takes a state and validates it with `SdfState::validate`, whose
docs list the rules. The model never edits the state, so a reader of an
`SdfMain` can trust every id it meets. Validation covers the references, the
finiteness of every number, and repeated names. Validation leaves the
arguments' values unchecked. Numbers stay `f64`, counts included.

An `Error` reports the broken rule with the entries involved. An `SdfEntryId`
displays as `table[index]`, which points to the entry in a document.
