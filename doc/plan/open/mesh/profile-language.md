# Profile Language

_Part of the [mesh plan](README.md)._

A profile is a named piece of configuration whose elements stand for
[`vxl mesh`](mesh.md) flags. `--profile` applies a profile whole, and
`--values-from` applies only a profile's values.
[Built-in profiles](#built-in-profiles) ship in the binary. Default profiles
like `--profile pbr` work before any `.vxlconfig` exists. The rest are
user-defined under `.vxlconfig`'s `mesh.profiles` key. A config profile sharing
a built-in's name replaces it wholesale. A profile can include another profile's
values with `valuesFrom`. Hyphenated profile names take camel-case value names
because `-` is subtraction in the [value language](value-language.md): a
`metallic-smoothness` profile would bake `metallicSmoothness`.

## Schema

Profiles are written as jsonc; the block below gives their shape in TypeScript
notation, and the doc comments tie each element to the flag it mirrors.
[Loading](#loading) holds the checks that enforce the schema.

```ts
/** The `.vxlconfig` shape `vxl mesh` reads. */
interface VxlConfig {
  /** The `mesh` command's slice of `.vxlconfig`. */
  mesh?: {
    /** The user-defined profiles, by name. */
    profiles?: Record<string, Profile>;
  };
}

/** A fragment of the value program, one or more `name = expr` bindings. */
type Bindings = string;

/** An expression of the value language. */
type Expr = string;

/** A file name, `{file-stem}` replaced before use. */
type FileTemplate = string;

/** The writer flags' `<linear | srgb>` argument. */
type Transfer = "linear" | "srgb";

/** A profile; each element mirrors a `vxl mesh` flag. */
interface Profile {
  /** Mirrors `--values-from` per entry; writers never travel. */
  valuesFrom?: string[];

  /** Mirrors `--compute-index`, each domain key holding its bound name. */
  computeIndex?: {
    corner?: string;
    face?: string;
    swatch?: string;
    voxel?: string;
  };

  /** Mirrors `--compute-occlusion`, the value the bound name. */
  computeOcclusion?: string;

  /** Mirrors `--compute-voxel-position`, the value the bound name. */
  computeVoxelPosition?: string;

  /** Mirrors `--value` per entry. */
  values?: Bindings[];

  /** Mirrors `--voxel-size`. */
  voxelSize?: number;

  /** Mirrors `--method`. */
  method?: "culled" | "greedy" | "naive";

  /** Mirrors `--texture-shape`, a number the flag's `<n>` form. */
  textureShape?: "fit" | "line" | "pot" | "square" | number;

  /** The files written beside the mesh. */
  files?: {
    /** Mirrors `--write-file-png-value` per template. */
    png?: Record<FileTemplate, { transfer: Transfer; value: Expr }>;

    /** Mirrors `--write-file-json-value` per template and key. */
    json?: Record<
      FileTemplate,
      Record<string, { transfer: Transfer; value: Expr }>
    >;
  };

  /** Mirrors `--material-count` by length; omitted, count 0. */
  materials?: MaterialEntry[];

  /** Mirrors `--primitive` per entry; omitted, the implicit primitive. */
  primitives?: PrimitiveEntry[];

  /** Mirrors the `--write-mesh-extra-*` flags, one entry per name. */
  meshExtras?: Record<string, ExtraEntry>;
}

/** A material, its list position the flags' `<material-index>`. */
interface MaterialEntry {
  /** Mirrors `--material-name`. */
  name?: string;

  /** Mirrors `--material-uv` per entry; omitted, the list derives from use. */
  uvs?: ("corner" | "face" | "swatch" | "voxel")[];

  /** Mirrors the `--write-material-slot-*` flags, a property per key. */
  slots?: Record<string, SlotEntry>;

  /** Mirrors the `--write-material-extra-*` flags, one entry per name. */
  extras?: Record<string, ExtraEntry>;
}

/** A slot's source, the kind naming the `--write-material-slot-*` tail. */
type SlotEntry =
  | { kind: "value"; value: Expr }
  | { kind: "file"; file: FileTemplate };

/** An extras entry, the kind naming the extras flag's tail. */
type ExtraEntry =
  | { kind: "image-file"; file: FileTemplate }
  | { kind: "image-value"; value: Expr; transfer: Transfer }
  | { kind: "json-file"; file: FileTemplate }
  | { kind: "json-value"; value: Expr; transfer: Transfer };

/** A primitive, its list position the flags' `<primitive-index>`. */
interface PrimitiveEntry {
  /** Mirrors `--primitive-name`. */
  name?: string;

  /** Mirrors the `--primitive` select argument; omitted, `"true"`. */
  select?: Expr;

  /** Mirrors the `--primitive` material index; omitted, `none`. */
  material?: number;

  /** Mirrors `--write-primitive-normal`; omitted, `true`. */
  normal?: boolean;

  /** Mirrors `--write-primitive-uv` per entry; omitted, the material's list. */
  uvs?: ("corner" | "face" | "swatch" | "voxel")[];

  /** Mirrors `--write-primitive-builtin-value` per attribute. */
  builtins?: Record<string, Expr>;

  /** Mirrors `--write-primitive-custom-value` per underscore name. */
  customs?: Record<string, { value: Expr; transfer: Transfer }>;
}
```

The defaults mirror the flags', and an empty `materials` or `primitives` list
counts as omitted; see
[Primitives and materials](mesh.md#primitives-and-materials) for the implicit
primitive and [UV streams](mesh.md#uv-streams) for the derived `uvs` lists.

### Example

The block below shows the schema as a jsonc example: a placeholder marks where
a profile writes its names and expressions.

```jsonc
{
  "mesh": {
    "profiles": {
      "<name>": {
        "valuesFrom": ["<profile>"],
        "values": ["<name> = <expr>"],
        "computeIndex": { "<domain>": "<dst-name>" },
        "computeOcclusion": "<dst-name>",
        "computeVoxelPosition": "<dst-name>",

        "voxelSize": 1.0,
        "method": "<culled | greedy | naive>",
        "textureShape": "<fit | line | pot | square | n>",

        "files": {
          "png": {
            "<template>": {
              "transfer": "<linear | srgb>",
              "value": "<expr>",
            },
          },
          "json": {
            "<template>": {
              "<key>": {
                "transfer": "<linear | srgb>",
                "value": "<expr>",
              },
            },
          },
        },

        "materials": [
          {
            "name": "<name>",
            "uvs": ["swatch", "face"],
            "slots": {
              "<property>": { "kind": "value", "value": "<expr>" },
            },
            "extras": {
              "<name>": {
                "kind": "image-value",
                "value": "<expr>",
                "transfer": "<linear | srgb>",
              },
            },
          },
        ],

        "primitives": [
          {
            "name": "<name>",
            "select": "<expr>",
            "material": 0,
            "normal": true,
            "uvs": ["swatch", "face"],
            "builtins": { "<ATTRIBUTE>": "<expr>" },
            "customs": {
              "<_NAME>": {
                "value": "<expr>",
                "transfer": "<linear | srgb>",
              },
            },
          },
        ],

        "meshExtras": {
          "<name>": {
            "kind": "json-value",
            "value": "<expr>",
            "transfer": "<linear | srgb>",
          },
        },
      },
    },
  },
}
```

## Profile values

A profile's values come from its `values` list, its `valuesFrom` list, and the
`computeIndex`, `computeOcclusion`, and `computeVoxelPosition` keys. The compute
keys define new values. The mirrored [`vxl mesh` flags](mesh.md#options) define
how bindings join the [program](value-language.md#programs). `valuesFrom`
imports append depth-first in list order, ahead of the profile's own values. A
profile imported twice lands once.

## Loading

The profiles resolve as a three-layer stack: the built-ins, then `~/.vxlconfig`,
then `<git-root>/.vxlconfig`. Each profile name is read from the last layer that
supplies it, wholesale. The layers merge into one namespace before `valuesFrom`
resolves, so a repo that overrides `defaults` changes every profile built on it,
including one from the home config.

The checks split by when they run:

1. every `.vxlconfig` load
   1. the file parsing
2. the first profile load
   1. the schema's shape, an unknown key erroring rather than skipping
3. each loaded profile
   1. its `values` fragments parsing
   2. its `valuesFrom` names resolving without a cycle
4. the profile applied whole
   1. its remaining expressions parsing
   2. every `material` inside the material count
   3. every `uvs` entry `corner`, `face`, `swatch`, or `voxel` named once
   4. no element claiming one destination twice
5. the run
   1. dimensions and shapes against the effective palette
   2. slot names and their encodings against the resolved output format
   3. name bindings against the command line

## Built-in profiles

The built-ins are the bottom layer of the [stack](#loading). The binary embeds
this section's map as data and parses it with the config deserializer when a
profile loads: the built-ins take the same schema by construction:

```jsonc
{
  // The glTF spec defaults, a mixin every profile builds on. Each
  // entry shadows its property with a defaulted copy.
  "defaults": {
    "values": [
      "baseColorFactor = default(baseColorFactor, rgba(1, 1, 1, 1))",
      "occlusionStrength = default(occlusionStrength, 1)",
      "roughnessFactor = default(roughnessFactor, 1)",
      "metallicFactor = default(metallicFactor, 1)",
      "emissiveFactor = default(emissiveFactor, rgb(0, 0, 0))",
      "emissiveStrength = default(emissiveStrength, 1)",
    ],
  },

  "albedo": {
    "valuesFrom": ["defaults"],
    "values": ["albedo = baseColorFactor"],
    "materials": [
      {
        "slots": {
          "baseColorTexture": { "kind": "value", "value": "albedo" },
        },
      },
    ],
  },

  // One value may fill several slots.
  "orm": {
    "valuesFrom": ["defaults"],
    "values": ["orm = rgb(occlusionStrength, roughnessFactor, metallicFactor)"],
    "materials": [
      {
        "slots": {
          "occlusionTexture": { "kind": "value", "value": "orm" },
          "metallicRoughnessTexture": { "kind": "value", "value": "orm" },
        },
      },
    ],
  },

  // white pins emissiveFactor against glTF's black default.
  "emissive": {
    "valuesFrom": ["defaults"],
    "values": [
      "maxStrength = max(emissiveStrength)",
      "emissive = emissiveFactor * emissiveStrength / max(maxStrength, 0.001)",
      "white = rgb(1, 1, 1)",
    ],
    "materials": [
      {
        "slots": {
          "emissiveTexture": { "kind": "value", "value": "emissive" },
          "emissiveFactor": { "kind": "value", "value": "white" },
          "emissiveStrength": { "kind": "value", "value": "maxStrength" },
        },
      },
    ],
  },

  // Writers never travel with valuesFrom, so pbr is no bundle: it
  // imports the three profiles' values and writes the whole material itself.
  "pbr": {
    "valuesFrom": ["albedo", "orm", "emissive"],
    "materials": [
      {
        "slots": {
          "baseColorTexture": { "kind": "value", "value": "albedo" },
          "occlusionTexture": { "kind": "value", "value": "orm" },
          "metallicRoughnessTexture": { "kind": "value", "value": "orm" },
          "emissiveTexture": { "kind": "value", "value": "emissive" },
          "emissiveFactor": { "kind": "value", "value": "white" },
          "emissiveStrength": { "kind": "value", "value": "maxStrength" },
        },
      },
    ],
  },
}
```

Every profile spells its own defaults through the `defaults` mixin, so a profile
never fails on a missing property: a property no layer supplies, or a material
that leaves it unset, takes the spec default the mixin names. A hand-written
`--value` gets no such guarantee because nothing auto-defaults.

`emissive` scales each material's color into `[0, 1]` of the palette's
strongest strength and sends that strength to the `emissiveStrength` slot, so
absolute brightness survives the 8-bit image. Packing the unbounded strength raw
would error on the first material above 1.

## User-defined profiles

Every other profile lives under `.vxlconfig`'s `mesh.profiles` key, in the same
schema, and may build on the built-ins. Seven examples follow:

1. `mse` packs metallic, smoothness, and normalized emissive strength into one
   mask
2. `orm-files` flips a built-in's slots from embedding to referencing
3. `heat` feeds a runtime of your own through the material extras
4. `palette` writes the [palette pattern](mesh.md#palettes) whole, rows beside
   their index
5. `vertex-colors` skips textures and rides base color on the vertices
6. `baked-ao` bakes [computed occlusion](value-language.md#computed-occlusion)
   into the standard slot
7. `glow-split` routes the glowing swatches to a second material through
   [primitives](mesh.md#primitives-and-materials)

```jsonc
{
  "mesh": {
    "profiles": {
      // emissiveStrength is unbounded, so the mask normalizes by the
      // palette's strongest strength and the raw intensity rides the
      // material slot.
      "mse": {
        "valuesFrom": ["defaults"],
        "values": [
          "smoothness = 1 - roughnessFactor",
          "maxStrength = max(emissiveStrength)",
          "mse = rgb(metallicFactor, smoothness, emissiveStrength / max(maxStrength, 0.001))",
        ],
        "files": {
          "png": {
            "{file-stem}-mse.png": { "transfer": "linear", "value": "mse" },
          },
        },
        "materials": [
          {
            "slots": {
              "emissiveStrength": { "kind": "value", "value": "maxStrength" },
            },
          },
        ],
      },

      // kind file references the written png where kind value would
      // embed.
      "orm-files": {
        "valuesFrom": ["orm"],
        "files": {
          "png": {
            "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" },
          },
        },
        "materials": [
          {
            "slots": {
              "occlusionTexture": {
                "kind": "file",
                "file": "{file-stem}-orm.png",
              },
              "metallicRoughnessTexture": {
                "kind": "file",
                "file": "{file-stem}-orm.png",
              },
            },
          },
        ],
      },

      // A per-swatch heat mask and one accent color, entries a
      // runtime of your own looks up under extras.vxl.values. The
      // heat png writes and its extra references it; the plain accent
      // inlines its numbers.
      "heat": {
        "valuesFrom": ["defaults"],
        "values": [
          ["heat", "step(0.001, emissiveStrength)"],
          ["accent", "avg(baseColorFactor.rgb)"],
        ],
        "files": {
          "png": {
            "{file-stem}-heat.png": { "transfer": "linear", "value": "heat" },
          },
        },
        "materials": [
          {
            "extras": {
              "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" },
              "accent": {
                "kind": "json-value",
                "value": "accent",
                "transfer": "srgb",
              },
            },
          },
        ],
      },

      // The palette pattern: rows under the mesh's extras.vxl.values,
      // the index they are read by on the primitive, no material at
      // all.
      "palette": {
        "valuesFrom": ["albedo"],
        "computeIndex": { "swatch": "swatchIndex" },
        "primitives": [
          {
            "customs": {
              "_PALETTE": { "value": "u8(swatchIndex)", "transfer": "linear" },
            },
          },
        ],
        "meshExtras": {
          "albedo": {
            "kind": "json-value",
            "value": "albedo",
            "transfer": "linear",
          },
        },
      },

      // No textures: base color rides the vertices as COLOR_0, no
      // material at all.
      "vertex-colors": {
        "valuesFrom": ["albedo"],
        "primitives": [{ "builtins": { "COLOR_0": "albedo" } }],
      },

      // Occlusion floored at 0.2 and baked whole into the standard
      // slot: a corner texture, the corner UV stream deriving. It
      // reads no palette property, so no defaults mixin.
      "baked-ao": {
        "computeOcclusion": "computedOcclusion",
        "values": ["ao = max(computedOcclusion, 0.2)"],
        "materials": [
          {
            "slots": {
              "occlusionTexture": { "kind": "value", "value": "ao" },
            },
          },
        ],
      },

      // Two materials, two primitives: the solid swatches drawn plain,
      // the glowing swatches with the emissive surface. valuesFrom
      // emissive supplies maxStrength, emissive, and white.
      "glow-split": {
        "valuesFrom": ["emissive"],
        "values": [
          "glowing = emissiveStrength > 0",
          "solid = !glowing",
          "albedo = baseColorFactor",
        ],
        "materials": [
          {
            "name": "body",
            "slots": {
              "baseColorTexture": { "kind": "value", "value": "albedo" },
            },
          },
          {
            "name": "glow",
            "slots": {
              "baseColorFactor": { "kind": "value", "value": "white" },
              "emissiveTexture": { "kind": "value", "value": "emissive" },
              "emissiveStrength": { "kind": "value", "value": "maxStrength" },
            },
          },
        ],
        "primitives": [
          { "name": "body", "select": "solid", "material": 0 },
          { "name": "glow", "select": "glowing", "material": 1 },
        ],
      },
    },
  },
}
```

With the output `turret.glb`, `--profile glow-split` expands to

```sh
--value "maxStrength = max(emissiveStrength)"   # valuesFrom: emissive
--value "emissive = emissiveFactor * emissiveStrength / max(maxStrength, 0.001)"
--value "white = rgb(1, 1, 1)"
--value "glowing = emissiveStrength > 0"
--value "solid = !glowing"
--value "albedo = baseColorFactor"
--material-count 2
--material-name 0 body
--material-name 1 glow
--primitive 0 solid
--primitive 1 glowing
--primitive-name 0 body
--primitive-name 1 glow
--write-material-slot-value 0 baseColorTexture albedo
--write-material-slot-value 1 baseColorFactor white
--write-material-slot-value 1 emissiveTexture emissive
--write-material-slot-value 1 emissiveStrength maxStrength
```

with the defaults elided.

## Expansion

With the output `turret.glb`, `--profile orm` expands to

```sh
--value "occlusionStrength = default(occlusionStrength, 1)"   # defaults mixin
--value "roughnessFactor = default(roughnessFactor, 1)"
--value "metallicFactor = default(metallicFactor, 1)"
--value "orm = rgb(occlusionStrength, roughnessFactor, metallicFactor)"
--material-count 1
--write-material-slot-value 0 occlusionTexture orm          # slots: kind value
--write-material-slot-value 0 metallicRoughnessTexture orm
```

with the unused defaults elided. The
[`orm-files` variant](#user-defined-profiles) moves the image to a written file
the slots reference:

```sh
# the values as above
--write-file-png-value turret-orm.png orm linear              # files.png
--write-material-slot-file 0 occlusionTexture turret-orm.png  # slots: kind file
--write-material-slot-file 0 metallicRoughnessTexture turret-orm.png
```

Files take their names from `{file-stem}` templates, `--file-stem` replacing the
default, the output mesh's stem. A template spells its file name literally, so a
hyphenated profile keeps its hyphens: a `metallic-smoothness` profile writes
`turret-metallic-smoothness.png` even though the value it bakes is
`metallicSmoothness`.

An explicit flag beats the profile: a hand-written flag replaces the element
claiming its destination, wherever it sits on the line. Two hand-written flags
colliding stays the error it always was, and a hand flag naming a material or
primitive index the run never declared still errors rather than growing the
count. A material's `uvs` list is one element, so any `--material-uv` naming the
material replaces all of it, and a geometry flag replaces its key the same way:
`--method culled` beside a profile spelling `greedy` meshes culled.
