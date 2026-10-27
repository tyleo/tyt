import assert from "node:assert/strict";
import {
  intersect,
  smoothIntersect,
  smoothSubtract,
  smoothUnion,
  subtract,
  union,
} from "./booleans.ts";
import type { Entry } from "./entry.ts";
import { sphere } from "./primitives.ts";
import { circle } from "./primitives2d.ts";
import type { Shape2d } from "./shape2d.ts";
import type { Shape3d } from "./shape3d.ts";
import { shapes2d, shapes3d } from "./test/tables.ts";

type Combine = <S extends Shape3d | Shape2d>(a: S, b: S) => S;

const CASES: [string, Combine, Entry][] = [
  ["intersect", (a, b) => intersect(a, b), {
    kind: "intersect",
    shapes: [0, 1],
  }],
  ["smoothIntersect", (a, b) => smoothIntersect(0.1, a, b), {
    kind: "smoothIntersect",
    radius: 0.1,
    shapes: [0, 1],
  }],
  ["smoothSubtract", (a, b) => smoothSubtract(0.1, a, b), {
    kind: "smoothSubtract",
    radius: 0.1,
    base: 0,
    cutters: [1],
  }],
  ["smoothUnion", (a, b) => smoothUnion(0.1, a, b), {
    kind: "smoothUnion",
    radius: 0.1,
    shapes: [0, 1],
  }],
  ["subtract", (a, b) => subtract(a, b), {
    kind: "subtract",
    base: 0,
    cutters: [1],
  }],
  ["union", (a, b) => union(a, b), { kind: "union", shapes: [0, 1] }],
];

for (const [call, combine, entry] of CASES) {
  Deno.test(`${call} writes its 3D shapes`, () => {
    const a = sphere([0, 0, 0], 1);
    const b = sphere([1, 0, 0], 1);
    assert.deepStrictEqual(shapes3d(combine(a, b)), [
      { kind: "sphere", center: [0, 0, 0], radius: 1 },
      { kind: "sphere", center: [1, 0, 0], radius: 1 },
      entry,
    ]);
  });

  Deno.test(`${call} writes its 2D shapes`, () => {
    const a = circle([0, 0], 1);
    const b = circle([1, 0], 1);
    assert.deepStrictEqual(shapes2d(combine(a, b)), [
      { kind: "circle", center: [0, 0], radius: 1 },
      { kind: "circle", center: [1, 0], radius: 1 },
      entry,
    ]);
  });
}

Deno.test("a boolean's shapes share one dimension", () => {
  assert.throws(
    () => union(sphere([0, 0, 0], 1), circle([0, 0], 1) as never),
    { message: "union shapes[1] must be a Shape3d, not a Shape2d" },
  );
  assert.throws(
    () => subtract(circle([0, 0], 1), sphere([0, 0, 0], 1) as never),
    { message: "subtract cutters[0] must be a Shape2d, not a Shape3d" },
  );
});

Deno.test("a boolean without a first shape errors", () => {
  assert.throws(() => union(), {
    message: "union shapes must be at least one shape, not []",
  });
  assert.throws(() => smoothSubtract(0.1, 3 as never), {
    message: "smoothSubtract base must be a Shape3d or a Shape2d, not 3",
  });
});
