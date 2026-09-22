# Mesh plan

Status: **closed** (2026-09-21). The three tracks landed phase by phase on
`main`: vox-value-language as its own crate, voxsmith's `mesh` operation over
a record of the run, and vxl's `mesh` command with its flags, profiles, and
cascade. `mesh-old` is deleted on both sides, the raw geometry is a
crate-internal seam, and the [worked examples](examples.md) run as written.
The build bent from the design where the landed crates decided:

1. A `false` select writes an empty primitive, keeping primitive indices
   stable, and the bridge emits its zero-count accessors
2. A corner-domain select errors instead of routing, because a select routes
   whole faces
3. Two slots or extras share an embedded image by expression text and bake
   domain, so one value under two spellings embeds twice
4. A file-referenced texture bakes where its value sits, and a declared stream
   list has to hold that domain
5. A mesh image extra samples through a stream some primitive writes at its
   bake domain, and every primitive writing it places it at one position
6. JSON and extras floats narrow to `f32` after the transfer runs in `f64`
7. `srgb` on an unsigned value errors wherever it errors on a bool or a string
8. A vec3 `COLOR_0` lands as VEC4 with an alpha of one, the document's one
   vertex color shape
10. A profile with no `materials` declares count 0, and its compute keys
    travel with `--values-from`
11. The bridge fixes the rest of the output: `u32` indices, one sampler per
    texture, explicit defaults, key-ordered extras, and the frame change that
    sends the voxel `+Y` to glTF `-Z`

This plan rewrites [`vxl mesh`](mesh.md) which makes geometry and palette-atlas
textures. The plan keeps the geometry core and redoes everything around it: the
shipped map surface retires wholesale and will be deleted from the codebase. The
new surface is a small expression language. A material map is a value that you
write. A profile describes a full run in `.vxlconfig`. A profile can import
another's values. The plan also adds parts `vxl mesh` never had.

1. [`vxl mesh`](mesh.md): the command reference, from its arguments to the
   glTF a run emits.
2. [Value language](value-language.md): the expression language for material
   values. The results land in images, JSON files, and the mesh's own material.
3. [Profile language](profile-language.md): the profiles, defined in a new
   `.vxlconfig` file or built into the binary.
4. [Worked examples](examples.md): Examples showing a profile, its command line,
   and the glTF that it makes.
5. [Implementation](implementation.md): how the tool changes land, from the new
   language crate to the code the rewrite deletes.
