# Profile Language

_Part of the [voxel rendering plan](../../plan/open/voxel-rendering/README.md)._

A profile is a named piece of configuration whose elements stand for
[`vxl object render`](render.md) flags. `--profile` applies a profile whole,
`--views-from` applies only a profile's views, and `--lights-from` only its
light rig. Repeated `--profile` flags [stack](#stacking) their profiles.
[Built-in profiles](#built-in-profiles) ship in the binary, so
`--profile turnaround` works before any `.vxlconfig` exists. The rest are
user-defined under `.vxlconfig`'s `object.render.profiles` key. A config
profile sharing a built-in's name replaces it wholesale. A profile can import
another profile's views with `viewsFrom` and its rig with `lightsFrom`.

## Schema

Profiles are written as jsonc; the block below gives their shape in TypeScript
notation, and the doc comments tie each element to the flag it mirrors.
[Loading](#loading) holds the checks that enforce the schema.

```ts
/** The `.vxlconfig` shape `vxl object render` reads. */
interface VxlConfig {
  /** The `object` commands' slice of `.vxlconfig`. */
  object?: {
    /** The `object render` command's slice. */
    render?: {
      /** The user-defined profiles, by name. */
      profiles?: Record<string, Profile>;
    };
  };
}

/** A profile; each element mirrors a `vxl object render` flag. */
interface Profile {
  /** One line the profile listings print beside the name. */
  description?: string;

  /** Mirrors `--width`. */
  width?: number;

  /** Mirrors `--height`. */
  height?: number;

  /** Mirrors `--background`; omitted, transparent. */
  background?: "transparent" | string;

  /** Mirrors `--occlusion`; omitted, `corner`. */
  occlusion?: "none" | "corner";

  /** Mirrors `--voxel-size`, meters per voxel; omitted, `1`. */
  voxelSize?: number;

  /** Mirrors `--views-from` per entry; only the views travel. */
  viewsFrom?: string[];

  /** Mirrors the `--view-*` flags, keyed by the name that suffixes the file. */
  views?: Record<string, ViewEntry>;

  /** Mirrors `--lights-from` per entry; only the rig travels. */
  lightsFrom?: string[];

  /** Mirrors the `--light-*` flags, its list position the `<light-index>`. */
  lights?: LightEntry[];
}

type Vec3 = [number, number, number];
type Quat = [number, number, number, number];

/** A view. */
interface ViewEntry {
  /** Mirrors `--view-frame`, `--view-position`, and a rotation flag, or
   *  `--view-orbit` for the whole element. Omitted, the flags set it. */
  transform?: PoseTransform;

  /** Mirrors `--view-projection`; omitted, `perspective`. */
  projection?: "perspective" | "orthographic";

  /** Mirrors `--view-fov`, the vertical field of view; omitted, `35`.
   *  Errors under `orthographic`. */
  fov?: number;

  /** Mirrors `--view-scale`, the world units across the shorter image axis
   *  under `orthographic`; omitted, `fit`. Errors under `perspective`. */
  scale?: number;

  /** Mirrors `--view-select`, hierarchy-path globs; omitted, the rendered
   *  objects. The subject that `subject` and `orbit` are about. */
  select?: string[];
}

/** A light. Mirrors `--light <light-index> <kind>`. */
type LightEntry =
  | {
      kind: "directional";
      /** Mirrors `--light-frame` and a rotation flag. Omitted, the flags
       *  set it. */
      transform?: RotationTransform;
      /** Mirrors `--light-shadow`; omitted, `per-corner`. */
      shadow?: "none" | "per-pixel" | "per-face" | "per-corner";
      /** Mirrors `--light-color`; omitted, `#FFFFFF`. */
      color?: string;
      /** Mirrors `--light-strength`; omitted, `1`. */
      strength?: number;
    }
  | {
      kind: "point";
      /** Mirrors `--light-frame` and `--light-position`, or
       *  `--light-orbit` for the whole element. Omitted, the flags set it. */
      transform?: PositionTransform;
      shadow?: "none" | "per-pixel" | "per-face" | "per-corner";
      color?: string;
      strength?: number;
      /** Mirrors `--light-range`, meters; omitted, no cutoff. */
      range?: number;
    }
  | {
      kind: "hemisphere";
      /** Mirrors `--light-sky` and `--light-ground`; omitted, `#FFFFFF`. */
      sky?: string;
      ground?: string;
      strength?: number;
    };

/** A position and a rotation. Views, and spot lights later. */
type PoseTransform =
  | { kind: "world"; position: Vec3; rotation: Rotation }
  | { kind: "subject"; position: Vec3; rotation: Rotation }
  /** Degrees. `distance` omitted, `fit`. */
  | { kind: "orbit"; azimuth: number; elevation: number;
      distance?: number | "fit" }
  /** `path` mirrors `--view-node`. */
  | { kind: "node"; path: string; position: Vec3; rotation: Rotation };

/** A rotation only. A directional light sits at its frame's origin. */
type RotationTransform =
  | { kind: "world"; rotation: Rotation }
  | { kind: "camera"; rotation: Rotation }
  /** `path` mirrors `--light-node`. */
  | { kind: "node"; path: string; rotation: Rotation };

/** A position only. Point lights. */
type PositionTransform =
  | { kind: "world"; position: Vec3 }
  | { kind: "subject"; position: Vec3 }
  | { kind: "camera"; position: Vec3 }
  /** Degrees. `distance` is required. */
  | { kind: "orbit"; azimuth: number; elevation: number; distance: number }
  /** `path` mirrors `--light-node`. */
  | { kind: "node"; path: string; position: Vec3 };

/** Shared by every shape. Each form mirrors the `--view-*` and `--light-*`
 *  flag of its name. */
type Rotation =
  /** The form a node stores. */
  | { kind: "quaternion"; value: Quat }
  /** Fixed x, y, then z; `unit` omitted, `deg`. */
  | { kind: "euler"; value: Vec3; unit?: "deg" | "rad" }
  /** Aims -Z at a point in the frame; omitted, the frame's origin. */
  | { kind: "look-at"; target?: Vec3 }
  /** Azimuth from +Z toward +X, elevation toward +Y, facing the origin. */
  | { kind: "angles"; azimuth: number; elevation: number };
```

The defaults mirror the flags'. The frames and the rotation forms are the
[contract's transforms](contract.md#transforms), and a light's kind limits its
frames as `--light-frame` says. A profile entry can say what the flags cannot:
an `euler` rotation in radians, and a `look-at` with no `target`.

### Example

The block below shows the schema as a jsonc example: a placeholder marks where
a profile writes its names and values.

```jsonc
{
  "object": {
    "render": {
      "profiles": {
        "<name>": {
          "description": "<one line>",
          "width": 1024,
          "height": 1024,
          "background": "<transparent | #RRGGBB>",
          "occlusion": "<none | corner>",
          "voxelSize": 1.0,

          "viewsFrom": ["<profile>"],
          "views": {
            "<orbit-view>": {
              "transform": {
                "kind": "orbit",
                "azimuth": 45,
                "elevation": 30,
                "distance": "<fit | meters>",
              },
              "projection": "perspective",
              "fov": 35,
              "select": ["<glob>"],
            },
            "<posed-view>": {
              "transform": {
                "kind": "<world | subject>",
                "position": [0, 0, 10],
                "rotation": { "kind": "look-at", "target": [0, 0, 0] },
              },
              "projection": "orthographic",
              "scale": 8,
            },
            "<node-view>": {
              "transform": {
                "kind": "node",
                "path": "<glob>",
                "position": [0, 1, 5],
                "rotation": { "kind": "look-at" },
              },
            },
          },

          "lightsFrom": ["<profile>"],
          "lights": [
            {
              "kind": "directional",
              // `node` adds "path": "<glob>".
              "transform": {
                "kind": "<world | camera | node>",
                "rotation": { "kind": "angles", "azimuth": -30, "elevation": 30 },
              },
              "shadow": "<none | per-pixel | per-face | per-corner>",
              "color": "#FFFFFF",
              "strength": 2.5,
            },
            {
              "kind": "point",
              // `node` adds "path": "<glob>".
              "transform": {
                "kind": "<world | subject | camera | node>",
                "position": [0, 4, 0],
              },
              "range": 12,
            },
            {
              "kind": "hemisphere",
              "sky": "#9FB4CC",
              "ground": "#4A3E33",
              "strength": 1,
            },
          ],
        },
      },
    },
  },
}
```

A rotation takes any of its four forms wherever one appears above:

```jsonc
{ "kind": "quaternion", "value": [0, 0, 0, 1] }
{ "kind": "euler", "value": [0, 37, 0], "unit": "deg" }
{ "kind": "look-at", "target": [0, 0, 0] }
{ "kind": "angles", "azimuth": 45, "elevation": 30 }
```

## Views and lights

Views and lights are independent halves. A profile can set either. A run
whose stack and flags set no view renders the built-in `hero` view, and one
that sets no light uses the built-in `studio` rig, the way `object mesh` falls
back to its implicit primitive. `--profile turnaround --profile dusk` composes
a view set with a light rig.

A profile can import one half of another profile, the way a mesh profile's
`valuesFrom` imports values. `viewsFrom` imports views and `lightsFrom`
imports a rig:

1. Imports land depth-first in list order, ahead of the profile's views or
   rig
2. Only the imported half travels. The image elements and the other half
   stay behind
3. A profile's views and its rig each land once, however many imports and
   `--profile` flags bring them
4. Imports resolve after the cascade merges, so a config that overrides
   `hero` changes `turnaround`
5. An imported element collides like a stack member's. A rig is one
   element, so a profile's rig comes from its `lights` or from one import
6. An import cycle errors

## Loading

The profiles resolve as a cascade: the built-ins, then each `.vxlconfig` in
the order the [mesh implementation notes](../mesh/implementation.md#ty-preferences)
lay out, the user's `~/.vxlconfig` first and then every directory from the
git root down to the working directory. Each profile name is read from the
last layer that supplies it, wholesale. The layers merge into one namespace
before `viewsFrom` and `lightsFrom` resolve. `vxl profile object render list`
prints the merged namespace grouped by the file supplying each name, each
name beside its `description`.

The checks split by when they run:

1. every run
   1. each `.vxlconfig` parsing
   2. the schema's shape, an unknown key erroring rather than skipping
   3. each value's range: a positive `width`, `height`, `voxelSize`, `fov`,
      `scale`, `range`, and orbit distance, a `strength` of zero or more,
      and a `#RRGGBB` color
2. the profile stack applied whole
   1. every `--profile`, `--views-from`, and `--lights-from` name resolving,
      listed once per flag
   2. the imports resolving without a cycle
   3. the stack merging, an element two members set erroring
3. the record
   1. every view holding a transform, from the stack or the flags, and every
      directional or point light likewise
   2. each view's name non-empty and free of path separators
   3. a view's frame `world`, `subject`, or `node`, a directional light's
      `world`, `camera`, or `node`, and a node path only under the `node`
      frame, which needs one
   4. `fov` under `perspective` and below 180 degrees, and `scale` under
      `orthographic`
4. the render
   1. the selection matching objects, in a document holding some
   2. each view's `select` matching a rendered object
   3. each transform resolving: a look-at whose eye is its target errors, as
      does a `node` path matching no node path or several

## Built-in profiles

The built-ins are the bottom layer of the [cascade](#loading). The binary
embeds this section's map as data and parses it with the config deserializer
when a profile loads, so the built-ins take the same schema by construction:

```jsonc
{
  "hero": {
    "description": "One perspective view from the front-right-top",
    "views": {
      "hero": { "transform": { "kind": "orbit", "azimuth": 45, "elevation": 30 } },
    },
  },

  "front": {
    "description": "One view of the front",
    "views": {
      "front": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 0 } },
    },
  },
  "back": {
    "description": "One view of the back",
    "views": {
      "back": { "transform": { "kind": "orbit", "azimuth": 180, "elevation": 0 } },
    },
  },
  "left": {
    "description": "One view of the left side",
    "views": {
      "left": { "transform": { "kind": "orbit", "azimuth": -90, "elevation": 0 } },
    },
  },
  "right": {
    "description": "One view of the right side",
    "views": {
      "right": { "transform": { "kind": "orbit", "azimuth": 90, "elevation": 0 } },
    },
  },
  "top": {
    "description": "One view from above with the front at the bottom of the image",
    "views": {
      "top": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": 90 } },
    },
  },
  "bottom": {
    "description": "One view from below",
    "views": {
      "bottom": { "transform": { "kind": "orbit", "azimuth": 0, "elevation": -90 } },
    },
  },

  "turnaround": {
    "description": "The hero view and the four sides",
    "viewsFrom": ["hero", "front", "right", "back", "left"],
  },

  // The key light's offset gives a box three distinct shades. The shadow
  // takes the default granularity.
  "studio": {
    "description": "A key light from the upper left over a hemisphere light",
    "lights": [
      {
        "kind": "directional",
        "transform": {
          "kind": "camera",
          "rotation": { "kind": "angles", "azimuth": -30, "elevation": 30 },
        },
        "color": "#FFFFFF",
        "strength": 2.5,
      },
      {
        "kind": "hemisphere",
        "sky": "#9FB4CC",
        "ground": "#4A3E33",
        "strength": 1,
      },
    ],
  },

  // The strength of pi renders a white base color as white.
  "flat": {
    "description": "A shadowless headlight for judging color",
    "lights": [
      {
        "kind": "directional",
        "transform": {
          "kind": "camera",
          "rotation": { "kind": "angles", "azimuth": 0, "elevation": 0 },
        },
        "shadow": "none",
        "color": "#FFFFFF",
        "strength": 3.1416,
      },
    ],
  },
}
```

The seven view profiles each hold one orbit view at `fit`, named for the
profile, so `--profile front --profile top` renders `front` and `top` in one
run. `top` and `bottom` look along the up axis, where -Z is up, so the front
sits at the bottom of the image. `turnaround` imports five of them and holds
nothing else.

`studio` is one key light in the `camera` frame, from the upper left of
whoever is looking, over a hemisphere light. A light in the `camera` frame
follows every view, so each view of a turnaround lights the same way. The key
light sets no shadow granularity and takes `per-corner`. `flat` is a headlight
with no shadow, for judging color alone.

## User-defined profiles

Every other profile lives under `.vxlconfig`'s `object.render.profiles` key,
in the same schema, and may build on the built-ins. Three examples follow:

1. `dusk`: a low warm key light under a blue sky, a rig to compose with any
   view set
2. `iso`: an orthographic view from the front-left-top
3. `sheet`: `turnaround`'s views over `dusk`'s rig at 512 by 512 on a dark
   background, one profile for the whole run

```jsonc
{
  "object": {
    "render": {
      "profiles": {
        // The key light sits in the world frame, so it stays put as the
        // views orbit and the shadows fall one way across the turnaround.
        "dusk": {
          "description": "A low warm key light under a blue sky",
          "lights": [
            {
              "kind": "directional",
              "transform": {
                "kind": "world",
                "rotation": { "kind": "angles", "azimuth": 120, "elevation": 10 },
              },
              "color": "#FFB070",
              "strength": 2,
            },
            {
              "kind": "hemisphere",
              "sky": "#4060A0",
              "ground": "#201810",
              "strength": 0.8,
            },
          ],
        },

        "iso": {
          "description": "An orthographic view from the front-left-top",
          "views": {
            "iso": {
              "transform": { "kind": "orbit", "azimuth": -45, "elevation": 30 },
              "projection": "orthographic",
            },
          },
        },

        // Both halves imported; the profile adds the image elements alone.
        "sheet": {
          "description": "The turnaround under dusk at half size on a dark background",
          "width": 512,
          "height": 512,
          "background": "#202020",
          "viewsFrom": ["turnaround"],
          "lightsFrom": ["dusk"],
        },
      },
    },
  },
}
```

With the input `turret.voxj`, `--profile sheet` expands to

```sh
--width 512
--height 512
--background '#202020'
--view-orbit hero 45 30 fit      # viewsFrom: turnaround, then hero
--view-orbit front 0 0 fit
--view-orbit right 90 0 fit
--view-orbit back 180 0 fit
--view-orbit left -90 0 fit
--light 0 directional            # lightsFrom: dusk
--light-frame 0 world
--light-angles 0 120 10
--light-color 0 '#FFB070'
--light-strength 0 2
--light 1 hemisphere
--light-sky 1 '#4060A0'
--light-ground 1 '#201810'
--light-strength 1 0.8
```

with the defaults elided, and under `--to png` writes `turret-hero.png`
through `turret-left.png` beside the input.

An explicit flag beats the profile: a hand-written flag replaces the element
claiming its destination, wherever it sits on the line. Two hand-written flags
colliding stays the error it always was, and a `--light-*` flag naming an index
the rig never declared still errors rather than growing the rig. A view's
transform is one element, so a flag posing a view replaces a profile's orbit
only with a whole pose. A rig is one element too: any `--light` declaration
replaces the stack's rig whole, where the other `--light-*` flags fill the
rig's lights element by element, so `--light-shadow 0 per-pixel` over `studio`
changes the key light's shadow alone.

## Stacking

Repeating `--profile` stacks the profiles in line order, so

```sh
vxl object render turret.voxj
  --profile front
  --profile top
  --profile flat
```

renders `front` and `top` under the `flat` headlight without a profile for
that combination. The views merge by name, element by element, and the lights
as one rig. When two members set the same image element, the same element of
one view, or the rig, the run errors and reports both. `--views-from` and
`--lights-from` join the stack with one half each. The hand flags replace the
stack's elements as they replace one profile's.
