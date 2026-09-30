# Vxl Command-Line Reference

`vxl` is a command-line tool for working with voxel data. It converts between
voxel formats, meshes voxels into editable geometry, renders voxels into
images, voxelizes meshes, bakes material textures, and inspects and validates
voxel-json documents.

This reference targets the voxel-json format. Its on-disk shape, encodings,
palette model, hierarchy, and validation rules are defined in the
[voxel-json file format spec](../../../../projects/voxel-formats/voxj/docs/voxel-json-file-format.md).
The pages below link into that spec rather than restating it, and any rule here
must agree with it.

A voxel-json file comes in two interchangeable forms with identical content:
`.voxj` (plain UTF-8 JSON) and `.voxjz` (a zip archive holding one `.voxj`
member). Every command that reads a voxel file accepts either form, recognized
by leading bytes (`{` versus `PK`) rather than by extension, as the spec
requires in [File Extensions](../../../../projects/voxel-formats/voxj/docs/voxel-json-file-format.md#file-extensions).
The reference writes `.voxj` for brevity.

A document holds an ordered `palettes` array and a shared
`runtimeState.valuePools` array the palettes reference by index. Each palette
pairs an ordered set of `properties`, each naming a property
(`baseColor`, `metallic`, `roughness`, and so on) with a
value pool it draws from, and a `materials` table holding one row of
value-indices per material, one per property. A voxel samples one
material index per layer its object references through `layers`. Layers
combine by overriding: contributions apply in `layers` order, back to front,
and each property takes its value from the last layer that supplies it. This
model is defined in
[Palettes](../../../../projects/voxel-formats/voxj/docs/voxel-json-file-format.md#palettes).
The palette commands address a target their own way, described under
[`vxl palette`](reference/palette/README.md); property keys are the glTF
vocabulary names such as `baseColor`.

> Notation: `<required>`, `[optional]`, `[optional=default]`, and `flag` for a
> presence or settable boolean.

## Commands

Every command sits under the noun it addresses, then its verb.

- [`vxl vox-doc to <format>`](reference/vox-doc/to/README.md): convert between
  voxel formats, and the canonical way to re-encode, pack, and unpack a
  document.
- [`vxl vox-doc validate`](reference/vox-doc/validate.md): check a document
  against the spec.
- [`vxl vox-doc show`](reference/vox-doc/show.md): report a document's contents.
- [`vxl mesh-doc voxelize`](reference/mesh-doc/voxelize.md): mesh to voxel grid.
- [`vxl object mesh`](../../../ref/mesh/mesh.md): voxel to editable mesh, with
  material maps as textures or per-vertex attributes.
- [`vxl object render`](../../../ref/render/render.md): voxel to image, one
  per view, inline in the terminal or as PNGs.
- [`vxl object material`](reference/object/material.md): bake material maps
  only.
- [`vxl object`](../../closed/vxl-object-commands/README.md#vxl-object): edit
  object properties, placements, and copies.
- [`vxl object voxels quantize`](reference/object/voxels/quantize.md): reduce
  the materials an object's voxels sample without changing its palettes.
- [`vxl object downsample` and `vxl object upsample`](../vxl-resample-commands/README.md):
  change an object's grid resolution by a whole-number factor.
- [`vxl object voxels`](../../closed/vxl-object-commands/README.md#vxl-object-voxels):
  move voxels within an object's grid.
- [`vxl node`](../../closed/vxl-object-commands/README.md#vxl-node): edit nodes
  and hierarchy edges.
- [`vxl node list`](reference/node/README.md): print the scene graph.
- [`vxl palette`](reference/palette/README.md): list, show, quantize, and remap
  palettes.
- [`vxl profile object mesh list`](../../../ref/mesh/profile-language.md#loading):
  list the profiles `object mesh --profile` can apply.
- [`vxl profile object render list`](../../../ref/render/profile-language.md#loading):
  list the profiles `object render --profile` can apply.
- [`vxl profile object voxels quantize list`](reference/object/voxels/quantize.md#profiles):
  list the profiles `object voxels quantize --profile` can apply.
- [`vxl profile palette quantize list`](reference/palette/quantize.md#profiles):
  list the profiles `palette quantize --profile` can apply.
- [`vxl profile palette show list`](reference/palette/show.md#profiles): list
  the profiles `palette show --profile` can apply.

`vxl vox-doc to` already ships. The
[object commands plan](../../closed/vxl-object-commands/README.md) covers the
object, object voxels, and node commands. This plan covers the rest.

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
