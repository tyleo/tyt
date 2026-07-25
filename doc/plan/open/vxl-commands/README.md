# Vxl Command-Line Reference

`vxl` is a command-line tool for working with voxel data. It converts between
voxel formats, meshes voxels into editable geometry, voxelizes meshes, bakes
material textures, and inspects and validates voxel-json documents.

This reference targets the voxel-json format. Its on-disk shape, encodings,
palette model, hierarchy, and validation rules are defined in the
[voxel-json file format spec](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md).
The pages below link into that spec rather than restating it, and any rule here
must agree with it.

A voxel-json file comes in two interchangeable forms with identical content:
`.voxj` (plain UTF-8 JSON) and `.voxjz` (a zip archive holding one `.voxj`
member). Every command that reads a voxel file accepts either form, recognized
by leading bytes (`{` versus `PK`) rather than by extension, as the spec
requires in [File Extensions](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#file-extensions).
The reference writes `.voxj` for brevity.

A document holds an ordered `palettes` array and a shared
`runtimeState.valuePools` array the palettes reference by index. Each palette
pairs an ordered set of `properties`, each naming a property
(`baseColorFactor`, `metallicFactor`, `roughnessFactor`, and so on) with a
value pool it draws from, and a `materials` table holding one row of
value-indices per material, one per property. A voxel samples one
material index per layer its object references through `layers`. Layers
combine by overriding: contributions apply in `layers` order, back to front,
and each property takes its value from the last layer that supplies it. This
model is defined in
[Palettes](../../../../projects/voxel-codecs/voxj/docs/voxel-json-file-format.md#palettes).
The palette commands address a target their own way, described under
[`vxl palette`](reference/palette/README.md); property keys are the glTF names
such as `baseColorFactor`, not the old `rgba`.

> Notation: `<required>`, `[optional]`, `[optional=default]`, and `flag` for a
> presence or settable boolean.

## Commands

- [`vxl to <format>`](reference/to/README.md): convert between voxel formats,
  and the canonical way to re-encode, pack, and unpack a document.
- [`vxl mesh`](reference/mesh.md): voxel to editable mesh, with material maps as
  textures or per-vertex attributes.
- [`vxl material`](reference/material.md): bake material maps only.
- [`vxl voxelize`](reference/voxelize.md): mesh to voxel grid.
- [`vxl palette`](reference/palette/README.md): list, show, quantize, and remap
  palettes.
- [`vxl hierarchy show`](reference/hierarchy/README.md): print the scene graph.
- [`vxl validate`](reference/validate.md): check a document against the spec.
- [`vxl info`](reference/info.md): report a document's contents.

`vxl to` already ships. The rest are the subject of this plan.

## Cross-cutting

- [Conventions and cross-command options](reference/conventions.md): shared
  formats, defaults, settable booleans, palette addressing, the `--select` /
  `--select-index` object selectors, and repeating a flag for multiple values.
- [Design notes](reference/design-notes.md): rationale for the non-obvious
  choices, and future work.

## Implementation

- [Implementation checklist](checklist.md): the task list for building these
  commands. Start here when implementing.
- [Implementation decisions](reference/implementation-decisions.md): code-level
  decisions recorded as the commands are built, the Rust-level companion to the
  design notes.
