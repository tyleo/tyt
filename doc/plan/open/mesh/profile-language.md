# Profile Language

_Part of the [mesh plan](README.md)._

A profile is a named piece of configuration in one of two kinds. A
value profile is a reusable set of values, the mixin `--value-profile`
applies. An output profile is a run's whole surface, the geometry
options, materials, primitives, files, and extras `--output-profile`
expands into flags. Eight ship in the binary: the `defaults`, `albedo`, `orm`,
and `emissive` value profiles and the `albedo`, `orm`, `emissive`, and
`pbr` output profiles, so `--output-profile pbr` works before any
`.vxlconfig` exists. The rest are user-defined under `.vxlconfig`'s
`mesh.valueProfiles` and `mesh.outputProfiles` keys; the kinds are
separate namespaces, so one name may serve in both. A config profile
sharing a built-in's name replaces it wholesale, and extending a value
profile is a new name with `basedOn`. Hyphenated profile names take
camel-case value names, since `-` is subtraction in the
[value language](value-language.md): a `metallic-smoothness` profile
would bake `metallicSmoothness`.

## Value profiles

A value profile holds `values`, an optional `basedOn` list, and an
optional `computeOcclusion` key, and nothing else: it can define every
name a run needs and still writes nothing.
`--value-profile <profile>` applies the profile's values as if each
were a `--value` at the flag's own position, `basedOn` first:
depth-first in list order, every profile visited once, cycles an
error, the profile's own values last. So

```
--value a "0.5" --value-profile albedo --value b "a * 2"
```

defines `a`, then the `defaults` mixin `albedo` builds on, then
`albedo`'s own values, then `b`, and redefinition stays let-style
throughout, so a `--value` after the flag overrides a profile value
and every later expression sees the override.

A value profile requests
[computed occlusion](value-language.md#computed-occlusion) through its
`computeOcclusion` key, the flag's argument as its value, binding the
name the way `--compute-occlusion` does. Every request across
profiles and flags binds its name to the one computation, so
requests alias rather than collide. The binding rides `basedOn` with
the values, since an inherited expression needs its name.

## Output profiles

`--output-profile <profile>` applies an output profile, at most one
per run. The profile is the run's whole surface, the geometry options
ahead of an output shaped like the glTF it produces, and every
element expands to the flag it fires as:

```jsonc
"<name>": {
  // the geometry options, each key the flag it is named for;
  // omitted, a key leaves the flag's default
  "voxelSize": 1.0,
  "method": "<greedy | culled | naive>",
  "atlas": "<palette | unwrap>",
  "textureShape": "<fit | line | pot | square | n>",

  // value profiles applied first, in order, as if each were a
  // --value-profile at the --output-profile flag's position
  "values": ["<value-profile>"],

  // UV streams in TEXCOORD order, the --uv list; omitted, the list
  // derives from the domains the textures use
  "uvs": ["row", "face"],

  // files written beside the mesh; a file's transfer lives here and
  // nowhere else
  "files": {
    "png": {
      "<template>": { "transfer": "<linear | srgb>", "value": "<expr>" }
    },
    "json": {
      // the file's whole object, one { transfer, value } per key
      "<template>": {
        "<key>": { "transfer": "<linear | srgb>", "value": "<expr>" }
      }
    }
  },

  // one entry per material; the list's length is the material count,
  // the --material-count mirror
  "materials": [
    {
      "name": "<name>",   // optional, the glTF material.name
      "slots": {
        // kind value embeds or inlines an expression; kind file carries a
        // file field referencing a written file instead
        "<property>": { "kind": "value", "value": "<expr>" }
      },
      "extras": {
        // kinds image-file, image-value, json-file, json-value: the
        // -file kinds carry file, the -value kinds value and transfer
        "<name>": {
          "kind": "image-value",
          "value": "<expr>",
          "transfer": "<linear | srgb>"
        }
      }
    }
  ],

  // one entry per primitive, each hooked to a material by index
  "primitives": [
    {
      "name": "<name>",       // optional, rides the primitive's extras
      "select": "<expr>",   // the selects partition the faces
      "material": 0,
      "builtins": { "<ATTRIBUTE>": "<expr>" },
      "customs": {
        "<_NAME>": { "value": "<expr>", "transfer": "<linear | srgb>" }
      },
      "indices": { "<_NAME>": "<u8 | u16>" }
    }
  ],

  // the mesh's own extras, the same four kinds as a material's
  "meshExtras": {
    "<name>": {
      "kind": "json-value",
      "value": "<expr>",
      "transfer": "<linear | srgb>"
    }
  }
}
```

The kinds are the flag grid's tails. A slot's `value` kind fires
`--write-material-slot-value`, the `file` kind
`--write-material-slot-file`; an extras entry's four kinds fire the
four extras flags, on the material or the mesh by where the entry
sits; a primitives entry fires `--primitive` in list order, its
`material` and `select` the flag's arguments, its `builtins` firing
`--write-primitive-builtin-value` tokenless like the flag, its
`customs` `--write-primitive-custom-value` with their transfers, and
its `indices` `--write-primitive-index` with their widths;
`files.png` and `files.json` entries fire
`--write-file-png-value` and `--write-file-json-value` per key; and
each geometry key fires the flag it is named for. The `uvs` list spells
the `--uv` flags in order. The defaults mirror the flags': an omitted
geometry key takes its flag's default, an omitted `materials` is one
bare material, an omitted `primitives` the implicit primitive holding
every face on material `0`, and an omitted `uvs` derives from use the
way the bare line does; see
[Primitives and materials](mesh.md#primitives-and-materials) and
[UV streams](mesh.md#uv-streams).

Three rules keep a profile honest. A `file` field names a file the
profile's own `files` writes, so a slot or extras reference always
has bytes behind it, and a foreign file stays a hand-written flag. A
transfer lives on the file entry alone, referencers carrying none, so
nothing can fight it; a referenced png still cross-checks its
transfer against each slot's fixed encoding, the
`--write-material-slot-file` rule spelled in config, and no built-in
writes a file, so none can hit it. And the destination dicts make a
double claim unspellable where a key is the destination, the
remaining cross-dict collisions erroring at load: one custom
attribute spelled in both `customs` and `indices`, or one template
under both `png` and `json`.

An explicit flag beats the profile: a hand-written flag replaces the
element claiming its destination, wherever it sits on the line, while
two hand-written flags colliding stays the error it always was, and a
hand flag naming a material or primitive index the run never declared
still errors rather than growing the count. The `uvs` list is one
element, so any `--uv` flag replaces all of it, and a geometry flag
replaces its key the same way: `--method culled` beside a profile
spelling `greedy` meshes culled.

So with the output `turret.glb`, `--output-profile orm` expands to

```
--value occlusionStrength "default(occlusionStrength, 1)"   # values: orm
--value roughnessFactor "default(roughnessFactor, 1)"
--value metallicFactor "default(metallicFactor, 1)"
--value orm "rgb(occlusionStrength, roughnessFactor, metallicFactor)"
--write-material-slot-value 0 occlusionTexture orm          # slots: kind value
--write-material-slot-value 0 metallicRoughnessTexture orm
```

with the unused defaults elided. The
[`orm-files` variant](#user-defined-profiles) moves the image to a
written file the slots reference:

```
# the values as above
--write-file-png-value turret-orm.png orm linear              # files.png
--write-material-slot-file 0 occlusionTexture turret-orm.png  # slots: kind file
--write-material-slot-file 0 metallicRoughnessTexture turret-orm.png
```

Files take their names from `{file-stem}` templates, `--file-stem`
replacing the default, the output mesh's own stem. A template spells
its file name literally, so a hyphenated profile keeps its hyphens: a
`metallic-smoothness` profile writes `turret-metallic-smoothness.png`
even though the value it bakes is `metallicSmoothness`.

## Loading

User-defined profiles live in `.vxlconfig` files, one in the home
directory and one at the git root, read as jsonc: comments are stripped
ahead of the JSON parse, and a trailing comma stays the error strict
JSON makes it. The crate work behind the loading lives in the
[implementation notes](implementation.md#ty-preferences).

Each kind resolves as its own three-layer stack: the built-ins, then
`~/.vxlconfig`, then `<git-root>/.vxlconfig`. Each profile name is read
from the last layer that supplies it, wholesale, the rule the effective
palette already follows per property: a layer that respells a profile
respells all of it, so an override never inherits stray elements from
the layer below. The value-profile layers merge into one namespace
before `basedOn` resolves, so a repo that overrides `defaults` changes
every profile built on it, including one from the home config, and an
output profile's `values` list resolves against the same merged
namespace.

Loading checks every profile in the merged namespaces, so a broken
config fails the first run after the edit rather than the run that
first names the profile. Load-time checks are the ones that need no run
context: the schema's shape, every expression parsing, every `basedOn`
and `values` name resolving without a cycle, every `material` index
inside its profile's material count, every `file` reference naming a
written file, every `uvs` entry `row` or `face` named once, and no
element claiming one destination twice. The rest
wait for the run that decides them: dimensions and shapes need the
effective palette, slot names and their encodings need the resolved
output format, and name bindings need the command line.

## Built-in profiles

The built-ins are the bottom layer of the [stack](#loading). The
binary embeds this section's maps as data and parses them at startup
with the config deserializer, so the built-ins take the same schema by
construction and every run exercises the parse path. The value
profiles:

```jsonc
{
  // The glTF spec defaults, a mixin every profile builds on. Each
  // entry shadows its property with a defaulted copy.
  "defaults": {
    "values": [
      ["baseColorFactor", "default(baseColorFactor, rgba(1, 1, 1, 1))"],
      ["occlusionStrength", "default(occlusionStrength, 1)"],
      ["roughnessFactor", "default(roughnessFactor, 1)"],
      ["metallicFactor", "default(metallicFactor, 1)"],
      ["emissiveFactor", "default(emissiveFactor, rgb(0, 0, 0))"],
      ["emissiveStrength", "default(emissiveStrength, 1)"]
    ]
  },

  "albedo": {
    "basedOn": ["defaults"],
    "values": [["albedo", "baseColorFactor"]]
  },

  "orm": {
    "basedOn": ["defaults"],
    "values": [
      ["orm", "rgb(occlusionStrength, roughnessFactor, metallicFactor)"]
    ]
  },

  // white pins emissiveFactor against glTF's black default.
  "emissive": {
    "basedOn": ["defaults"],
    "values": [
      ["maxStrength", "max(emissiveStrength)"],
      [
        "emissive",
        "emissiveFactor * emissiveStrength / max(maxStrength, 0.001)"
      ],
      ["white", "rgb(1, 1, 1)"]
    ]
  }
}
```

and the output profiles, each pulling the value profiles it needs and
spelling one material. Every one omits `primitives`, taking the
implicit primitive that holds every face on material `0`, and embeds,
so a slot fixes each encoding and no entry carries a transfer:

```jsonc
{
  "albedo": {
    "values": ["albedo"],
    "materials": [
      {
        "slots": {
          "baseColorTexture": { "kind": "value", "value": "albedo" }
        }
      }
    ]
  },

  // One value may fill several slots.
  "orm": {
    "values": ["orm"],
    "materials": [
      {
        "slots": {
          "occlusionTexture": { "kind": "value", "value": "orm" },
          "metallicRoughnessTexture": { "kind": "value", "value": "orm" }
        }
      }
    ]
  },

  "emissive": {
    "values": ["emissive"],
    "materials": [
      {
        "slots": {
          "emissiveTexture": { "kind": "value", "value": "emissive" },
          "emissiveFactor": { "kind": "value", "value": "white" },
          "emissiveStrength": { "kind": "value", "value": "maxStrength" }
        }
      }
    ]
  },

  // Output profiles do not compose, so pbr is no bundle: it pulls the
  // three value profiles and spells the whole material itself.
  "pbr": {
    "values": ["albedo", "orm", "emissive"],
    "materials": [
      {
        "slots": {
          "baseColorTexture": { "kind": "value", "value": "albedo" },
          "occlusionTexture": { "kind": "value", "value": "orm" },
          "metallicRoughnessTexture": { "kind": "value", "value": "orm" },
          "emissiveTexture": { "kind": "value", "value": "emissive" },
          "emissiveFactor": { "kind": "value", "value": "white" },
          "emissiveStrength": { "kind": "value", "value": "maxStrength" }
        }
      }
    ]
  }
}
```

Every profile spells its own defaults through the `defaults` mixin,
which is what makes a profile never fail on a missing property: a
property no layer supplies, or a material that leaves it unset, takes
the spec default the mixin names. A hand-written `--value` gets no such
guarantee, since nothing auto-defaults.

`emissiveStrength` has a minimum of 0 and no maximum, so the `emissive`
profile and the `mse` example both normalize by the palette's strongest
strength and send that strength to the `emissiveStrength` slot. Packing
it raw would put an unbounded value in an 8-bit channel, which errors
on the first material above 1 rather than clamping. Normalizing is also
the convention the packed maps target, a `0..1` mask in the image and
the intensity on the material. The two agree on this deliberately, and
merging them is harmless, since each binds the slot to the same
`maxStrength`.

The emissive trio is the whole profile: the image carries each
material's color scaled into `0..1` of the palette's strongest
strength, the strength slot carries that strength back, so absolute
brightness survives a `0..1` image, and the white `emissiveFactor`
leaves the image untinted where glTF would otherwise multiply it by
black.

## User-defined profiles

Every other profile lives under `.vxlconfig`'s `mesh.valueProfiles`
and `mesh.outputProfiles` keys, in the same schema, and may build on
the built-ins. `mse` packs metallic, smoothness, and normalized
emissive strength into one mask; `orm-files` flips a built-in's slots
from embedding to referencing; `heat` feeds a runtime of your own
through the material extras; `palette` writes the
[palette pattern](mesh.md#palettes) whole, rows beside their index;
`vertex-colors` skips textures and rides base color on the vertices;
`baked-ao` bakes
[computed occlusion](value-language.md#computed-occlusion) into the
standard slot; `glow-split` routes the glowing rows to a second
material through
[primitives](mesh.md#primitives-and-materials):

```jsonc
{
  "mesh": {
    "valueProfiles": {
      "mse": {
        "basedOn": ["defaults"],
        "values": [
          ["smoothness", "1 - roughnessFactor"],
          ["maxStrength", "max(emissiveStrength)"],
          [
            "mse",
            "rgb(metallicFactor, smoothness, emissiveStrength / max(maxStrength, 0.001))"
          ]
        ]
      },

      "heat": {
        "basedOn": ["defaults"],
        "values": [
          ["heat", "step(0.001, emissiveStrength)"],
          ["accent", "avg(baseColorFactor.rgb)"]
        ]
      },

      // Occlusion averaged down to faces and floored at 0.2. It reads
      // no palette property, so no defaults mixin.
      "baked-ao": {
        "computeOcclusion": "computedOcclusion",
        "values": [["ao", "max(faceAverage(computedOcclusion), 0.2)"]]
      },

      // basedOn emissive supplies maxStrength, emissive, and white.
      "glow": {
        "basedOn": ["emissive"],
        "values": [
          ["glowing", "emissiveStrength > 0"],
          ["solid", "!glowing"],
          ["albedo", "baseColorFactor"]
        ]
      }
    },

    "outputProfiles": {
      // emissiveStrength is unbounded, so the mask normalizes by the
      // palette's strongest strength and the raw intensity rides the
      // material slot.
      "mse": {
        "values": ["mse"],
        "files": {
          "png": {
            "{file-stem}-mse.png": { "transfer": "linear", "value": "mse" }
          }
        },
        "materials": [
          {
            "slots": {
              "emissiveStrength": { "kind": "value", "value": "maxStrength" }
            }
          }
        ]
      },

      // kind file references the written png where kind value would
      // embed.
      "orm-files": {
        "values": ["orm"],
        "files": {
          "png": {
            "{file-stem}-orm.png": { "transfer": "linear", "value": "orm" }
          }
        },
        "materials": [
          {
            "slots": {
              "occlusionTexture": { "kind": "file", "file": "{file-stem}-orm.png" },
              "metallicRoughnessTexture": {
                "kind": "file",
                "file": "{file-stem}-orm.png"
              }
            }
          }
        ]
      },

      // A per-row heat mask and one accent color, entries a
      // runtime of your own looks up under extras.vxl.values. The
      // heat png writes and its extra references it; the plain accent
      // inlines its numbers.
      "heat": {
        "values": ["heat"],
        "files": {
          "png": {
            "{file-stem}-heat.png": { "transfer": "linear", "value": "heat" }
          }
        },
        "materials": [
          {
            "extras": {
              "heat": { "kind": "image-file", "file": "{file-stem}-heat.png" },
              "accent": {
                "kind": "json-value",
                "value": "accent",
                "transfer": "srgb"
              }
            }
          }
        ]
      },

      // The palette pattern: rows under the mesh's extras.vxl.values,
      // the index they are read by on the primitive, and a bare
      // material the runtime shades past.
      "palette": {
        "values": ["albedo"],
        "materials": [{}],
        "primitives": [
          { "material": 0, "indices": { "_PALETTE": "u8" } }
        ],
        "meshExtras": {
          "albedo": {
            "kind": "json-value",
            "value": "albedo",
            "transfer": "linear"
          }
        }
      },

      // No textures: base color rides the vertices as COLOR_0.
      "vertex-colors": {
        "values": ["albedo"],
        "materials": [{}],
        "primitives": [
          { "material": 0, "builtins": { "COLOR_0": "albedo" } }
        ]
      },

      // Computed occlusion into the standard slot, the face UV stream
      // deriving; the value profile binds the name itself.
      "baked-ao": {
        "values": ["baked-ao"],
        "materials": [
          {
            "slots": {
              "occlusionTexture": { "kind": "value", "value": "ao" }
            }
          }
        ]
      },

      // Two materials, two primitives: the solid rows drawn plain,
      // the glowing rows with the emissive surface.
      "glow-split": {
        "values": ["glow"],
        "materials": [
          {
            "name": "body",
            "slots": {
              "baseColorTexture": { "kind": "value", "value": "albedo" }
            }
          },
          {
            "name": "glow",
            "slots": {
              "baseColorFactor": { "kind": "value", "value": "white" },
              "emissiveTexture": { "kind": "value", "value": "emissive" },
              "emissiveStrength": { "kind": "value", "value": "maxStrength" }
            }
          }
        ],
        "primitives": [
          { "name": "body", "select": "solid", "material": 0 },
          { "name": "glow", "select": "glowing", "material": 1 }
        ]
      }
    }
  }
}
```

With the output `turret.glb`, `--output-profile glow-split` expands to

```
--value maxStrength "max(emissiveStrength)"   # values: glow, basedOn emissive
--value emissive "emissiveFactor * emissiveStrength / max(maxStrength, 0.001)"
--value white "rgb(1, 1, 1)"
--value glowing "emissiveStrength > 0"
--value solid "!glowing"
--value albedo "baseColorFactor"
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
