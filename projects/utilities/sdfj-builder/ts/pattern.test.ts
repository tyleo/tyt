import assert from "node:assert/strict";
import type { Entry } from "./entry.ts";
import { material } from "./material.ts";
import {
  bands,
  bricks,
  cells,
  checker,
  gradient,
  grain,
  noise,
  type Pattern,
  speckle,
} from "./pattern.ts";
import { add } from "./step.ts";
import { ball, black, document, white } from "./test/tables.ts";

const gray = material({ baseColor: "#808080" });

const CASES: [string, () => Pattern, Entry][] = [
  [
    "bands",
    () => bands([white, black], 0.1, { axis: "x", warp: 0.05, seed: 1 }),
    {
      kind: "bands",
      materials: [0, 1],
      axis: "x",
      period: 0.1,
      warp: 0.05,
      seed: 1,
    },
  ],
  [
    "bricks",
    () =>
      bricks([white, black], 0.2, 0.05, {
        seed: 9,
        axis: "z",
        border: { material: gray, width: 0.01 },
      }),
    {
      kind: "bricks",
      materials: [0, 1],
      length: 0.2,
      height: 0.05,
      axis: "z",
      seed: 9,
      border: { material: 2, width: 0.01 },
    },
  ],
  [
    "cells",
    () =>
      cells([white, black], 0.15, {
        seed: 7,
        border: { material: gray, width: 0.02 },
      }),
    {
      kind: "cells",
      materials: [0, 1],
      size: 0.15,
      seed: 7,
      border: { material: 2, width: 0.02 },
    },
  ],
  ["checker", () => checker([white, black], 0.05), {
    kind: "checker",
    materials: [0, 1],
    size: 0.05,
  }],
  [
    "gradient",
    () =>
      gradient([white, black], {
        axis: "y",
        from: 0,
        to: 1,
        warp: 0.1,
        seed: 2,
      }),
    {
      kind: "gradient",
      materials: [0, 1],
      axis: "y",
      from: 0,
      to: 1,
      warp: 0.1,
      seed: 2,
    },
  ],
  [
    "grain",
    () => grain([white, black], 0.05, { axis: "y", warp: 0.03, seed: 3 }),
    {
      kind: "grain",
      materials: [0, 1],
      axis: "y",
      period: 0.05,
      warp: 0.03,
      seed: 3,
    },
  ],
  ["noise", () => noise([white, black], 0.1, { octaves: 3, seed: 4 }), {
    kind: "noise",
    materials: [0, 1],
    scale: 0.1,
    octaves: 3,
    seed: 4,
  }],
  [
    "speckle",
    () => speckle(white, [black, gray], 0.05, { density: 0.1, seed: 5 }),
    {
      kind: "speckle",
      base: 0,
      accents: [1, 2],
      size: 0.05,
      density: 0.1,
      seed: 5,
    },
  ],
];

for (const [call, pattern, entry] of CASES) {
  Deno.test(`${call} writes its materials and options`, () => {
    const { patterns } = document([add("shape", ball, pattern())]);
    assert.deepStrictEqual(patterns, [entry]);
  });
}

Deno.test("grain with one base writes the materials of shades(base)", () => {
  const pattern = grain(white, 0.05, { axis: "y", seed: 1 });
  const { materials, patterns, shades } = document([
    add("shape", ball, pattern),
  ]);
  assert.deepStrictEqual(materials, [
    { kind: "material", properties: { baseColor: "#FFFFFF" } },
    { kind: "shade", shades: 0, index: 0 },
    { kind: "shade", shades: 0, index: 1 },
    { kind: "shade", shades: 0, index: 2 },
  ]);
  assert.deepStrictEqual(shades, [{ base: 0, count: 3 }]);
  assert.deepStrictEqual(patterns, [
    { kind: "grain", materials: [1, 2, 3], axis: "y", period: 0.05, seed: 1 },
  ]);
});

Deno.test("speckle with one accent writes a list of one accent", () => {
  const pattern = speckle(white, black, 0.05, { density: 0.1, seed: 1 });
  const { patterns } = document([add("shape", ball, pattern)]);
  assert.deepStrictEqual(patterns, [
    {
      kind: "speckle",
      base: 0,
      accents: [1],
      size: 0.05,
      density: 0.1,
      seed: 1,
    },
  ]);
});

Deno.test("a cells border takes a material and a width", () => {
  assert.throws(() => cells([white], 0.1, { seed: 1, border: gray as never }), {
    message: "cells border must be an object, not a Material",
  });
  assert.throws(
    () => cells([white], 0.1, { seed: 1, border: { material: gray } as never }),
    { message: "cells border width must be a finite number, not undefined" },
  );
});

Deno.test("a pattern's materials have to be materials", () => {
  assert.throws(() => checker([white, "#000000" as never], 0.05), {
    message: 'checker materials[1] must be a Material, not "#000000"',
  });
});
