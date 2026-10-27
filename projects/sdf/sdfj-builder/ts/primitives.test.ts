import assert from "node:assert/strict";
import type { Entry } from "./entry.ts";
import {
  box,
  boxFrame,
  capsule,
  cone,
  cylinder,
  ellipsoid,
  halfSpace,
  octahedron,
  pyramid,
  roundCone,
  sphere,
  torus,
} from "./primitives.ts";
import type { Shape3d } from "./shape3d.ts";
import { shapes3d } from "./test/tables.ts";

const CASES: [string, () => Shape3d, Entry][] = [
  ["box", () => box([0, 0, 0], [1, 2, 3], { round: 0.1 }), {
    kind: "box",
    min: [0, 0, 0],
    max: [1, 2, 3],
    round: 0.1,
  }],
  ["boxFrame", () => boxFrame([0, 0, 0], [1, 2, 3], 0.1), {
    kind: "boxFrame",
    min: [0, 0, 0],
    max: [1, 2, 3],
    thickness: 0.1,
  }],
  ["capsule", () => capsule([0, 0, 0], [0, 1, 0], 0.25), {
    kind: "capsule",
    a: [0, 0, 0],
    b: [0, 1, 0],
    radius: 0.25,
  }],
  ["cone", () => cone([0, 0, 0], [0, 1, 0], 0.5, 0), {
    kind: "cone",
    a: [0, 0, 0],
    b: [0, 1, 0],
    radiusA: 0.5,
    radiusB: 0,
  }],
  ["cylinder", () => cylinder([0, 0, 0], [0, 1, 0], 0.5, { round: 0.1 }), {
    kind: "cylinder",
    a: [0, 0, 0],
    b: [0, 1, 0],
    radius: 0.5,
    round: 0.1,
  }],
  ["ellipsoid", () => ellipsoid([0, 0, 0], [1, 2, 3]), {
    kind: "ellipsoid",
    center: [0, 0, 0],
    radii: [1, 2, 3],
  }],
  ["halfSpace", () => halfSpace("+y", 0.25), {
    kind: "halfSpace",
    side: "+y",
    at: 0.25,
  }],
  ["octahedron", () => octahedron([0, 1, 0], 0.5), {
    kind: "octahedron",
    center: [0, 1, 0],
    radius: 0.5,
  }],
  ["pyramid", () => pyramid([0, 0, 0], 1, 2), {
    kind: "pyramid",
    baseCenter: [0, 0, 0],
    width: 1,
    height: 2,
  }],
  ["roundCone", () => roundCone([0, 0, 0], [0, 1, 0], 0.5, 0.25), {
    kind: "roundCone",
    a: [0, 0, 0],
    b: [0, 1, 0],
    radiusA: 0.5,
    radiusB: 0.25,
  }],
  ["sphere", () => sphere([1, 2, 3], 0.5), {
    kind: "sphere",
    center: [1, 2, 3],
    radius: 0.5,
  }],
  ["torus", () => torus([0, 0, 0], 1, 0.25, { axis: "x", from: -90, to: 90 }), {
    kind: "torus",
    center: [0, 0, 0],
    ringRadius: 1,
    tubeRadius: 0.25,
    axis: "x",
    from: -90,
    to: 90,
  }],
];

for (const [call, shape, entry] of CASES) {
  Deno.test(`${call} writes its arguments`, () => {
    assert.deepStrictEqual(shapes3d(shape()), [entry]);
  });
}

Deno.test("an option left out stays out of the entry", () => {
  assert.deepStrictEqual(shapes3d(torus([0, 0, 0], 1, 0.25, { axis: "x" })), [
    {
      kind: "torus",
      center: [0, 0, 0],
      ringRadius: 1,
      tubeRadius: 0.25,
      axis: "x",
    },
  ]);
  assert.deepStrictEqual(
    shapes3d(box([0, 0, 0], [1, 1, 1], { round: undefined })),
    [
      { kind: "box", min: [0, 0, 0], max: [1, 1, 1] },
    ],
  );
});

Deno.test("an argument the document cannot hold errors", () => {
  assert.throws(() => sphere([0, 0, 0], NaN), {
    message: "sphere radius must be a finite number, not NaN",
  });
  assert.throws(() => box([0, 0] as never, [1, 1, 1]), {
    message: "box min must be a Vec3 of 3 finite numbers, not [0, 0]",
  });
  assert.throws(() => halfSpace("y" as never, 0), {
    message:
      'halfSpace side must be one of "+x", "-x", "+y", "-y", "+z", "-z", not "y"',
  });
  assert.throws(() => box([0, 0, 0], [1, 1, 1], { rond: 0.1 } as never), {
    message: 'box takes no option "rond"',
  });
});
