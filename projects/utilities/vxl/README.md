# vxl

Works with voxels.

## Install

### Rust

[rustup](https://rustup.rs) installs Rust and Cargo. On Windows, the rustup site offers `rustup-init.exe` in place of this command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### vxl

vxl installs from [crates.io](https://crates.io/crates/vxl):

```sh
cargo install vxl --features bin
```

The `bin` feature builds the `vxl` binary. The same command upgrades vxl.

### Node

`vxl sdf-doc build` runs model files under [Node](https://nodejs.org) 24 or later. `--runtime bun` and `--runtime deno` run the model files under [Bun](https://bun.sh) or [Deno](https://deno.com) instead. The skill's passes run under Node.

### Agent skill

The `vxl-model` skill teaches [Claude Code](https://claude.com/claude-code) or [Codex](https://developers.openai.com/codex) to build voxel models with vxl. You can install it in your repository for Claude Code:

```sh
vxl integration skill vxl-model install claude
```

Or for Codex:

```sh
vxl integration skill vxl-model install agents
```

`--user` installs under the home directory instead:

```sh
vxl integration skill vxl-model install claude --user
```

The skill needs [Rust](#rust), [vxl](#vxl), and [Node](#node) installed.

### Shell completions

`vxl integration completion <shell> install` writes completions for bash, elvish, fish, powershell, or zsh:

```sh
vxl integration completion zsh install
```

For elvish, powershell, and zsh the install also prints one line to add to the shell's startup file. `vxl integration completion <shell> print` prints the completions to stdout instead.

## Modeling with Agents

A session with the [vxl-model](#agent-skill) skill writes a model file, voxelizes it, reviews the renders, and revises the model until it matches the prompt. Each model below came from a fresh session given only the prompt:

**Ramen stand**

> /vxl-model make a cyberpunk voxel ramen stand with a neon sign that says ramen

<p align="center">
  <img src="docs/readme/ramen-stand/hero.png" alt="A cyberpunk voxel ramen stand under a pink RAMEN neon sign" width="80%">
  <br>
  <img src="docs/readme/ramen-stand/front.png" alt="Ramen stand, front view" width="32%"> <img src="docs/readme/ramen-stand/right.png" alt="Ramen stand, right view" width="32%"> <img src="docs/readme/ramen-stand/top.png" alt="Ramen stand, top view" width="32%">
</p>

**Treasure chest**

> /vxl-model make a voxel treasure chest with the lid open and gold coins inside

<p align="center">
  <img src="docs/readme/chest/hero.png" alt="An open voxel treasure chest full of gold coins" width="80%">
  <br>
  <img src="docs/readme/chest/front.png" alt="Treasure chest, front view" width="32%"> <img src="docs/readme/chest/right.png" alt="Treasure chest, right view" width="32%"> <img src="docs/readme/chest/top.png" alt="Treasure chest, top view" width="32%">
</p>

**Fish tank**

> /vxl-model make a voxel fish tank with fish, seaweed, gravel, bubbles, and a little castle inside

<p align="center">
  <img src="docs/readme/fish-tank/hero.png" alt="A voxel fish tank with a castle, seaweed, and fish" width="80%">
  <br>
  <img src="docs/readme/fish-tank/front.png" alt="Fish tank, front view" width="32%"> <img src="docs/readme/fish-tank/right.png" alt="Fish tank, right view" width="32%"> <img src="docs/readme/fish-tank/top.png" alt="Fish tank, top view" width="32%">
</p>

**Candelabra**

> /vxl-model make a voxel iron candelabra with a twisted square stem and five lit candles on curling arms

<p align="center">
  <img src="docs/readme/candelabra/hero.png" alt="A voxel iron candelabra with five lit candles" width="80%">
  <br>
  <img src="docs/readme/candelabra/front.png" alt="Candelabra, front view" width="32%"> <img src="docs/readme/candelabra/right.png" alt="Candelabra, right view" width="32%"> <img src="docs/readme/candelabra/top.png" alt="Candelabra, top view" width="32%">
</p>

**Dragon**

> /vxl-model model a voxel dragon curled up asleep on a pile of gold with its wings folded

<p align="center">
  <img src="docs/readme/dragon/hero.png" alt="A red voxel dragon asleep on a heap of gold" width="80%">
  <br>
  <img src="docs/readme/dragon/front.png" alt="Dragon, front view" width="32%"> <img src="docs/readme/dragon/right.png" alt="Dragon, right view" width="32%"> <img src="docs/readme/dragon/top.png" alt="Dragon, top view" width="32%">
</p>

**Pirate ship**

> /vxl-model make a voxel pirate ship interior below deck with cannons, hammocks, barrels, and a hanging lantern

<p align="center">
  <img src="docs/readme/pirate-ship/hero.png" alt="A voxel pirate ship gun deck with cannons, barrels, and a lantern" width="80%">
  <br>
  <img src="docs/readme/pirate-ship/right.png" alt="Pirate ship, right view" width="32%"> <img src="docs/readme/pirate-ship/aft.png" alt="Pirate ship, aft view" width="32%"> <img src="docs/readme/pirate-ship/hammocks.png" alt="Pirate ship, hammocks view" width="32%">
</p>

Each session leaves a `.voxj` file beside its model file. You can render it to a PNG:

```sh
vxl object render ramen-stand.voxj --to png
```

And export it as a glb:

```sh
vxl object mesh ramen-stand.voxj --profile pbr
```

A permission rule in your project's `.claude/settings.json` lets Claude Code run vxl without asking each time:

```json
{
  "permissions": {
    "allow": ["Bash(vxl:*)"]
  }
}
```

`vxl sdf-doc build` runs the model file as a program, so the rule also lets the session run the code it writes.

## Editing

The `vxl object`, `vxl node`, `vxl palette edit`, and `vxl palette quantize` commands edit a document. Each reads any supported voxel format and writes Voxel Json beside the input by default. `vxl object mesh` writes a mesh instead, and `vxl node list` only prints the scene graph.

`--select` takes a gitignore-style hierarchy-path pattern. `--select-index` takes an id, an `a-b` range, or `*`. Both repeat and pick what the command acts on. `--select-parent` and `--select-parent-index` pick the one node at the parent end of an edge.

```sh
# Reads Voxel Max and writes scene.voxj beside it.
vxl object remove scene.vmax --select 'debris/**'
```

## Objects

`vxl object` commands write object properties and move the voxels along when needed:

1. `set origin` moves an object's grid within its node
2. `set edit-bounds` and `trim` resize the grid without moving a voxel in the scene
3. `link` and `unlink` add or drop one placement
4. `duplicate` copies objects within the document, and `add` copies them from another file along with their palettes
5. `downsample` and `upsample` change the grid's resolution by a whole-number factor per axis, scaling `origin` and `bounds` with it so `vxl node set scale` keeps the object's size in the scene
6. `mesh` writes the selected objects as a glTF mesh, with flags the [mesh reference](../../../doc/ref/mesh/mesh.md) covers

`vxl object render` draws the selected objects into one image per view, placed as `vxl object mesh` places them. The views and lights come from profiles in `.vxlconfig` and the flags that mirror them. With neither, the `hero` view renders under the `studio` lights. `--profile glow` adds a bloom over the emissive materials. The image shows inline in the terminal, or `--to png` writes one PNG per view beside the input. The [render reference](../../../doc/ref/render/render.md) covers the flags and the profiles.

```sh
# Copies the first three props under the one node matching house.
vxl object add scene.voxj --source props.voxj --select-index 0-2 --select-parent house

# Grows the crate's edit box from [-2 -6 0]..[2 2 4] by 2 voxels on -x and -z.
vxl object set edit-bounds scene.voxj --select crate --min -4 -6 -2 --max 2 2 4

# Merges the crate's voxels two per axis, keeping a block at least half live.
vxl object downsample scene.voxj --select crate --factor 2

# Splits each of the crate's voxels into ten per axis.
vxl object upsample scene.voxj --select crate --factor 10

# Shows the crate from the front-right-top under the studio lights.
vxl object render scene.voxj --select crate

# Writes scene-front.png and scene-top.png beside the input.
vxl object render scene.voxj --profile front --profile top --to png
```

## Object Voxels

`vxl object voxels` commands edit voxels within the grid and never change `origin` or `bounds`. `translate` shifts the voxels, `flip` mirrors them, and `rotate` turns them in quarter turns that follow the right-hand rule. One or three turns need the two turned dimensions to be equal. `vxl node set rotation` turns any object.

`vxl object voxels quantize` rewrites the materials voxels sample so each quantized layer samples at most `--max-materials` materials of its palette. The palettes stay as they are. Each object clusters apart unless `--shared` clusters the whole selection together. The clustering flags match `vxl palette quantize`'s.

```sh
# Turns the 6 x 8 x 6 crate a quarter turn about y.
vxl object voxels rotate scene.voxj --select crate --axis y --turns 1

# Brings the crate down to 16 materials, keeping metals and dielectrics apart.
vxl object voxels quantize scene.voxj --select crate --max-materials 16 --partition metallic
```

## Palettes

`vxl palette list` and `vxl palette show` print palettes. `vxl palette edit` and `vxl palette quantize` act on each palette `--index` selects. `--index` takes an id, an `a-b` range, or `*`. The flag repeats and defaults to `*`.

`vxl palette edit` runs a [value-language](../../../doc/ref/mesh/value-language.md) program over each palette. Each property the program reads enters as a swatch array holding one entry per material. `--value` adds bindings to the program. `--write-property` writes an expression's result into a property and adds the property when the palette lacks it. Materials sharing a value that are written alike keep sharing it, so a Voxel Max material slot stays one slot. A bare whole number written to a float property reads as `f64`. Every selected palette evaluates before any write lands, so an error leaves the document untouched.

`vxl palette quantize` reduces each palette to at most `--max-materials` materials and snaps every voxel sampling it. Each cluster collapses onto its most-sampled material, and a merged voxel takes that whole material. Clustering runs on `--property`, `baseColor` by default. `--partition` keeps materials apart unless they agree on a property. Materials no voxel samples drop. A palette no voxel samples errors.

`vxl palette edit` and both quantize commands take `--profile`, which applies flags saved in a `.vxlconfig` under `palette.edit.profiles`, `palette.quantize.profiles`, or `object.voxels.quantize.profiles`. `vxl profile palette edit list`, `vxl profile palette quantize list`, and `vxl profile object voxels quantize list` print the profiles.

```sh
# Gives rusty materials roughness 0.9 in every palette.
vxl palette edit scene.voxj \
  --value 'rust = tag == "rust"' \
  --write-property roughness 'mix(roughness, 0.9, rust)'

# Reduces every palette to Voxel Max's 255 colors.
vxl palette quantize scene.voxj --max-materials 255
```

## Nodes

`vxl node list` prints the scene graph. The other `vxl node` commands write nodes and hierarchy edges:

1. `set` writes a node's name, position, rotation, or scale. `set rotation` takes Euler angles in the order `vxl node list --show-transforms` prints them, in degrees by default
2. `add` creates an empty node
3. `link` and `unlink` add or drop one child edge
4. `remove` releases nodes along with each descendant left without a parent

Without a parent selector, `add`, `link`, and `unlink` act on the root list.

```sh
# Swings the door node 37 degrees about y.
vxl node set rotation scene.voxj --select house/door --rotation 0 37 0

# Places the same door node under garage too.
vxl node link scene.voxj --select house/door --select-parent garage
```

## Models

`vxl sdf-doc build` records a voxel model written in TypeScript as an `.sdfj` document. `vxl sdf-doc voxelize` samples the document into Voxel Json. The [TypeScript API](https://github.com/tyleo/tyt/blob/main/projects/utilities/sdfj-builder/docs/typescript-api.md) lists every call a model file can make, and the [modeling API](docs/modeling-api.md) sets how vxl runs a model. [Modeling with Claude Code](#modeling-with-claude-code) sets up the skill that runs these commands for you.

A build output named `.sdfjz` holds the document zipped, the way `.voxjz` holds Voxel Json. `voxelize` accepts either form, as does a library file a `.vxlconfig` lists.

```sh
# Records table.ts as table.sdfj with the built-in materials.
vxl sdf-doc build table.ts --library materials

# Writes table.voxj at 2.5 cm per voxel, then prints the report and writes it to table-report.txt.
vxl sdf-doc voxelize table.sdfj --voxel-size 0.025 --report

# Writes table.glb with the materials baked into textures.
vxl object mesh table.voxj --profile pbr
```
