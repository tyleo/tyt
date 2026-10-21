import assert from "node:assert/strict";
import { libraryGlobals } from "./library.ts";
import { material } from "./material.ts";
import { modelList } from "./model_list.ts";
import { part } from "./part.ts";
import { putApiInScope } from "./put_api_in_scope.ts";
import { sphere } from "./primitives.ts";
import { sdfjDocument, type SdfjNames } from "./sdfj_document.ts";
import { sdfjJson } from "./sdfj_json.ts";
import { add } from "./step.ts";
import {
  ball,
  BALL_ENTRY,
  black,
  document,
  library,
  white,
} from "./test/tables.ts";

const OAK = { baseColor: "#8A5A2B", roughness: 0.8 };

const BARK = { baseColor: "#4A3020" };

const LEAF = { baseColor: "#3A7A2A" };

/** `names` with an empty map for each table it lacks. */
function names(names: Partial<SdfjNames>): SdfjNames {
  return {
    shapes3d: {},
    shapes2d: {},
    materials: {},
    patterns: {},
    steps: {},
    parts: {},
    ...names,
  };
}

Deno.test("the chair writes the sdfj format's chair", async () => {
  putApiInScope([library("materials", [["oak", material(OAK)]])]);
  const chair = await import("./test/chair.ts");
  assert.deepStrictEqual(sdfjJson(sdfjDocument("chair", modelList(chair))), {
    version: 1,
    shapes3d: [
      {
        kind: "lathe",
        points: [[0.0375, 0], [0.03, 0.15], [0.045, 0.225], [0.03, 0.425]],
      },
      { kind: "translate", shape: 0, offset: [0.2, 0, 0.2] },
      { kind: "mirror", shape: 1, axes: "xz" },
      { kind: "box", min: [-0.25, 0.425, -0.25], max: [0.25, 0.475, 0.25] },
    ],
    materials: [
      { kind: "material", properties: OAK },
      { kind: "shade", shades: 0, index: 0 },
      { kind: "shade", shades: 0, index: 1 },
      { kind: "shade", shades: 0, index: 2 },
    ],
    shades: [{ base: 0, count: 3 }],
    patterns: [
      { kind: "grain", materials: [1, 2, 3], axis: "y", period: 0.05, seed: 1 },
      { kind: "grain", materials: [1, 2, 3], axis: "x", period: 0.05, seed: 2 },
    ],
    steps: [
      { kind: "add", name: "legs", shape: 2, pattern: 0 },
      { kind: "add", name: "seat", shape: 3, pattern: 1 },
    ],
    objects: [{ name: "chair", steps: [0, 1] }],
    nodes: [{ name: "chair", childObjects: [0], childNodes: [] }],
    rootNodes: [0],
    names: { materials: { oak: 0 } },
  });
});

Deno.test("the forest writes the sdfj format's forest", async () => {
  putApiInScope([
    library("materials", [
      ["bark", material(BARK)],
      ["leaf", material(LEAF)],
    ]),
  ]);
  const forest = await import("./test/forest.ts");
  assert.deepStrictEqual(sdfjJson(sdfjDocument("forest", modelList(forest))), {
    version: 1,
    shapes3d: [
      { kind: "cylinder", a: [0, 0, 0], b: [0, 0.5, 0], radius: 0.05 },
      { kind: "sphere", center: [0, 0.65, 0], radius: 0.25 },
      { kind: "ellipsoid", center: [0, 0.05, 0], radii: [0.05, 0.05, 0.075] },
    ],
    materials: [
      { kind: "material", properties: BARK },
      { kind: "material", properties: LEAF },
      { kind: "material", properties: { baseColor: "#C8B8A0" } },
    ],
    steps: [
      { kind: "add", name: "trunk", shape: 0, material: 0 },
      { kind: "add", name: "crown", shape: 1, material: 1 },
      { kind: "add", name: "body", shape: 2, material: 2 },
    ],
    objects: [
      { name: "tree", steps: [0, 1] },
      { name: "rabbit", steps: [2] },
    ],
    nodes: [
      { name: "tree", childObjects: [0], childNodes: [] },
      { name: "rabbit", childObjects: [1], childNodes: [] },
      {
        name: "rabbit.1",
        offset: [0.15, 0, 0.15],
        childObjects: [],
        childNodes: [1],
      },
      {
        name: "tree.1",
        offset: [-0.75, 0, 0],
        childObjects: [],
        childNodes: [0, 2],
      },
      {
        name: "tree.2",
        offset: [0, 0, -0.5],
        childObjects: [],
        childNodes: [0],
      },
      {
        name: "rabbit.2",
        offset: [-0.15, 0, 0.15],
        childObjects: [],
        childNodes: [1],
      },
      {
        name: "tree.3",
        offset: [0.75, 0, 0],
        childObjects: [],
        childNodes: [0, 5],
      },
      { name: "forest", childObjects: [], childNodes: [3, 4, 6] },
    ],
    rootNodes: [7],
    names: { materials: { bark: 0, leaf: 1 } },
  });
});

Deno.test("a shape feeding several steps writes one entry", () => {
  const { shapes3d, steps } = document([
    add("first", ball, white),
    add("second", ball, white),
  ]);
  assert.deepStrictEqual(shapes3d, [BALL_ENTRY]);
  assert.deepStrictEqual(steps, [
    { kind: "add", name: "first", shape: 0, material: 0 },
    { kind: "add", name: "second", shape: 0, material: 0 },
  ]);
});

Deno.test("a value built twice writes one entry", () => {
  const leg = () => part("leg", {}, [add("leg", sphere([0, 0, 0], 1), white)]);
  const { shapes3d, steps, objects } = document([leg(), leg()]);
  assert.deepStrictEqual(shapes3d, [BALL_ENTRY]);
  assert.deepStrictEqual(steps, [
    { kind: "add", name: "leg", shape: 0, material: 0 },
  ]);
  assert.deepStrictEqual(objects, [
    { name: "leg", steps: [0] },
    { name: "leg", steps: [0] },
  ]);
});

Deno.test("without interning a value built twice writes two entries", () => {
  const { shapes3d } = sdfjDocument(
    "model",
    [
      add("first", sphere([0, 0, 0], 1), white),
      add("second", sphere([0, 0, 0], 1), white),
    ],
    [],
    false,
  );
  assert.deepStrictEqual(shapes3d, [BALL_ENTRY, BALL_ENTRY]);
});

Deno.test("an empty model writes only its root node", () => {
  assert.deepStrictEqual(document([]).nodes, [
    { name: "model", childObjects: [], childNodes: [] },
  ]);
});

Deno.test("each named export writes its entry under its name", () => {
  const seat = add("seat", ball, white);
  const arm = part("arm", {}, [add("arm", ball, black)]);
  const written = sdfjDocument("model", [seat], [
    ["arm", arm],
    ["seat", seat],
    ["snow", white],
    ["white", white],
  ]);
  assert.deepStrictEqual(written.materials, [
    { kind: "material", properties: { baseColor: "#FFFFFF" } },
    { kind: "material", properties: { baseColor: "#000000" } },
  ]);
  assert.deepStrictEqual(written.rootNodes, [0]);
  assert.deepStrictEqual(
    written.names,
    names({
      shapes3d: {},
      materials: { snow: 0, white: 0 },
      steps: { seat: 0 },
      parts: { arm: 1 },
    }),
  );
});

Deno.test("two different entries taking one name errors", () => {
  const { mat } = libraryGlobals([library("woods", [["oak", white]])]);
  assert.throws(
    () => sdfjDocument("model", [add("seat", ball, mat.oak)], [["oak", black]]),
    { message: 'two different Materials take the name "oak"' },
  );
});
