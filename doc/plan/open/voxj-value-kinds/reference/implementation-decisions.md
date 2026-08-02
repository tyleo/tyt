# voxj value kinds implementation decisions

Code-level decisions made while executing the
[checklist](../checklist.md), recorded as they land. The plan-level
decisions and their rationale live in the [README](../README.md); this log
is for the finer implementation choices a reviewer of the Rust would want
explained. The checklist already names four that must land here:

1. The sentinel serde module's name and exact surface (the scalar and
   array forms, and how the adjacently tagged derive attaches them).
2. The vocabulary range check: its home, its signature, whether it
   absorbs `gltf_attributes.rs`'s `scalar_range`, and what becomes of the
   voxelizer's `--out-of-range-factor` clamp policy.
3. What value flaw checking voxcore keeps once the wire owns the domain
   rules.
4. Whether `internal/mesh/mesh_material.rs`'s `emissive_factor` field
   keeps its name.

No entries yet.
