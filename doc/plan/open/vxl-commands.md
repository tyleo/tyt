# Vxl Command-Line Reference

`vxl` is a command-line tool for working with voxel data stored in voxel-json
files: meshing voxels into editable geometry, voxelizing meshes, and managing
palettes.

A voxel-json file comes in two interchangeable forms: `.voxj` (plain UTF-8 JSON)
and `.voxjz` (a zip archive holding one `.voxj` member). They carry identical
content, so every command that accepts a voxel file should accept either form,
detected by leading bytes (`{` vs `PK`) rather than extension. The reference
below writes `.voxj` for brevity.

A file can hold multiple **palettes** in a shared `palettes` array, and each
palette declares an ordered set of **attributes** (`rgba`, `metallic`,
`roughness`, ...) with one **cell** (a row of values) per entry. Palette commands
address a target with two values: `palette-index` selects a palette by its
position in that array (default `0`), and `palette-attribute` selects which
attribute key within it to operate on (default `rgba`). This addressing model is
shared by every `palette` subcommand.

> Conventions used below:
> `<required>` `[optional]` `[optional=default]` `flag` (presence-only)

## `vxl mesh fbx`

```
vxl mesh fbx <input-voxj> <output-fbx> <object-name> [options]
```

Triangulates a single voxel object from a `.voxj` file into an FBX mesh.

1. `--method` `greedy` | `culled` | `naive` (default `greedy`): meshing strategy (see below).
2. `--ambient-occlusion` (flag, default off): bakes Minecraft-style per-vertex AO darkening into vertex colors at concave junctions.
3. `--texture <output-png> <num-channels> <palette-index> <cell-0> ... <cell-n>`: writes a PNG that the FBX's UVs sample from. See notes.

**Meshing strategies**

- `greedy`: merges coplanar, same-color faces into the fewest possible quads. Lowest triangle count.
- `culled`: emits one quad per solid/empty boundary face, without merging them.
- `naive`: emits all six faces of every solid voxel, including hidden interior faces. Highest triangle count.

Triangle count grows `greedy` < `culled` < `naive`. Choose `culled` or `naive`
only when you need stable per-voxel topology (e.g. for further per-face editing).

**`--texture` sub-arguments**

- `num-channels`: number of color channels written to the PNG (`1`-`4`).
- `palette-index`: index of the palette to read from.
- `cell-0 ... cell-n`: indices of the palette **cells** to bake into the texture. A cell is a row in the palette's `data` (referenced by its row index), and the selected attribute's value for each cell becomes one texel the FBX's UVs sample.

> Warning: Even with "cell" now defined (a palette row index), this flag packs a
> filename, an integer, a palette index, and a variadic list into a single
> option, which is fragile to type and to parse. It also can't say _which_
> attribute it reads. See **Recommendations**.

## `vxl voxelize`

```
vxl voxelize <input-mesh> <output-voxj> <side-length>
```

Rasterizes a mesh into a voxel grid. This is the inverse of `vxl mesh fbx`.

1. `input-mesh`: source mesh file (e.g. `.fbx`, `.obj`, `.gltf`).
2. `output-voxj`: destination voxel file.
3. `side-length`: grid resolution in **voxels** along the longest axis; the other axes are sized to keep aspect and the result is fitted tight to `bounds`. (The format has no physical units: one unit is one voxel, and real-world scale comes from hierarchy-node transforms, so this can only mean a voxel count, not an edge size.)

## `vxl palette`

Parent group for palette operations. A `.voxj` file may contain multiple
palettes, each addressed by `palette-attribute` (default `rgba`) and
`palette-index` (default `0`).

## `vxl palette quantize`

```
vxl palette quantize <input-voxj> <output-voxj> <color-count> [palette-attribute=rgba] [palette-index=0] [options]
```

Reduces a palette to at most `color-count` distinct colors and rewrites voxel
indices to match.

1. `--method` `median-cut` | `octree` | `kmeans`: quantization algorithm (**no default set**; pick one, `median-cut` suggested).
2. `--space` `oklab` | `lab` | `rgb` (default `oklab`): distance metric used when clustering colors.
3. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error diffusion when snapping colors.

> Note: dithering voxels happens in 3D index space, not a 2D image; confirm
> that's intentional.

## `vxl palette remap`

```
vxl palette remap <input-voxj> <output-voxj> <target> [options]
```

Remaps each voxel to its nearest entry in a **target** palette.

1. `--space` `oklab` | `lab` | `rgb` (default `oklab`): distance metric for nearest-color search.
2. `--dither` `none` | `floyd-steinberg` | `ordered` (default `none`): error diffusion when remapping.

> Warning: The original spec had **no target**: nothing says _what_ to remap onto.
> Since voxel samples are cell indices into a palette, the target must be another
> palette: a `<target.voxj>` argument, or `--target-index` / `--target-attribute`
> selecting a palette already in the same file. Without it the command is
> undefined.

## `vxl palette show`

```
vxl palette show <input-voxj> [palette-attribute=rgba] [palette-index=0] [options]
```

Prints a palette to the terminal.

1. `palette-attribute` (default `rgba`): the attribute to show.
2. `palette-index` (default `0`): index of the palette to show.
3. `--format` `rgba-color` | `grayscale-color` | `string` (default `rgba-color`): `*-color` prints colored swatches; `string` prints raw values.

> `--format string` should define its output (hex? `r,g,b,a`?) since that's
> what people pipe into scripts. Also note the `*-color` swatch formats only make
> sense for the `rgba` attribute; numeric attributes like `metallic` or
> `roughness` need a numeric/`string` rendering, so the sensible default for a
> non-color attribute isn't `rgba-color`.

## `vxl hierarchy show`

```
vxl hierarchy show <input-voxj>
```

Prints the scene graph contained in the file. Note this is a **DAG, not a
tree**: a node may have multiple parents (instancing), and only the nodes
listed in `rootHierarchyNodes` are roots, so `show` should make shared/instanced
nodes and unplaced library nodes visible rather than implying a strict tree.

# Recommendations

## Spec bugs (straight fixes)

- `quantize --method` has **no default**; set one (`median-cut` suggested).
- Typos: "deafult" in `remap`; pick one of **grey/gray** and use it consistently.
- `--texture num-channels` range is **`1`-`4`**, not `0`-`4` (zero channels is meaningless).

## Design smells

1. **`--texture` does too much.** Even with "cell" pinned down (a palette row
   index), one flag carrying a filename, an int, a palette index, and a variadic
   cell list is hard to type and fragile to parse, and it still can't name the
   attribute it bakes. Promote it to its own subcommand
   (`vxl mesh texture <input-voxj> <output-png> [--palette N] [--attribute rgba] [--cells 0..n]`)
   or read a small texture spec file.

2. **`quantize` has five positionals**, three of them optional-with-defaults.
   Optional positionals trailing required ones are order-fragile. Move
   `palette-attribute` / `palette-index` to `--attribute` / `--index` options,
   and use those **same two options** across `show`, `remap`, and `quantize` so
   the shared addressing model is expressed one way, not three.

3. **Inverse operations don't share naming logic.** `voxelize` (mesh to voxel) and
   `mesh fbx` (voxel to mesh) are inverses but look unrelated, and `mesh fbx` bakes
   the format into the command name (adding glTF/OBJ means `mesh gltf`,
   `mesh obj`, ...). Consider `vxl mesh export <in> <out> [--format fbx]` (infer
   from extension when omitted) paired with `vxl mesh import`, or keep
   `voxelize` but settle on one argument-ordering convention.

4. **`remap` has no target** (most serious gap). The command is undefined until
   it specifies _what_ it remaps onto.

5. **Boolean-as-value.** `--ambient-occlusion false|true` is un-idiomatic. Use a
   presence flag with `--ambient-occlusion` / `--no-ambient-occlusion`.

## Format-grounded gaps (from the `.voxj` spec)

These come directly from reading the file format, and several are stronger than
anything in the original CLI.

1. **No `vxl validate`.** The spec ships a long, precise validation checklist
   (index ranges, tight `bounds`, encoding byte lengths and zero pad bits,
   unique positions, sample arity, acyclic hierarchy, unit quaternions, edit-grid
   containment). That's begging for `vxl validate <file> [--json]`. It's the most
   valuable command the tool is missing.

2. **No way to re-encode / optimize.** The format defines three position
   encodings (`raw-json`, `bitmap-base64`, `hilbert-delta-varint-base64`) and
   three sample encodings (`raw-json`, `rle-json`, `packed-base64`), and
   explicitly says the right move is to build candidate pairs, compress each the
   way the file ships, and keep the smallest. A `vxl optimize <in> <out>`
   (or `vxl reencode --position ... --sample ...`) that picks the smallest pairing
   per object is a natural, high-value command. Re-encoding positions reorders
   voxels, so it must regenerate the sample channels to match. That is exactly
   the kind of invariant a CLI should own so authors don't get it wrong by hand.

3. **No `.voxjz` packing.** Since `.voxj` and `.voxjz` are interchangeable, add
   `vxl pack` / `vxl unpack` (or a `--compress` flag on writes) to convert
   between them. Today there's no way to produce the shipping form.

4. **`mesh fbx` only exports one object, by name.** The format describes a whole
   scene (a DAG of nodes with transforms, instancing, and roots), but `mesh fbx`
   meshes a single object and ignores all placement. There's no way to export the
   assembled scene. Consider a `--scene` mode (or separate command) that walks
   `rootHierarchyNodes` and bakes node transforms/instancing. Also, objects are
   canonically referenced **by index**, and nothing guarantees `name` is unique,
   so selecting by name is ambiguous; support `--object-index` (or accept either).

5. **Quantizing one attribute fractures multi-attribute cells.** A cell is a row
   across _all_ the palette's attributes (e.g. `rgba` + `metallic` + `roughness`).
   `quantize` reducing `color-count` on `rgba` alone has to decide what happens to
   cells that share a color but differ in `metallic`/`roughness`: merge them
   (losing the PBR distinction) or keep them split (so you don't actually hit
   `color-count`). The command needs to state this rule. Same concern for `remap`.

6. **`info` should report format internals.** Beyond voxel counts and bounds,
   `vxl info` can surface what's unique to this format: per-object position/sample
   encodings in use, palette attribute sets and cell counts, whether `editState`
   or `ext` namespaces are present, and instanced/unplaced nodes.

## Missing commands / features (general)

- **`vxl palette list`**: there's no way to discover which palette
  indices/attributes a file contains, yet every palette command demands one.
- **`--json` output mode** for `show` / `list` / `hierarchy` / `info` so they're
  scriptable.
- **stdin/stdout via `-`** so commands compose in pipelines.
- **Format inference from file extension** so users don't repeat themselves.
- **Dry-run / preview** for destructive palette ops (nice-to-have).
