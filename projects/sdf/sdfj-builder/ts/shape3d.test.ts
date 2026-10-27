import assert from "node:assert/strict";
import type { Entry } from "./entry.ts";
import type { Shape3d } from "./shape3d.ts";
import { ball, BALL_ENTRY, shapes3d } from "./test/tables.ts";

const CASES: [string, () => Shape3d, Entry][] = [
  ["bend", () => ball.bend("x", "+y", 2, [0, 1, 0]), {
    kind: "bend",
    shape: 0,
    along: "x",
    toward: "+y",
    radius: 2,
    pivot: [0, 1, 0],
  }],
  [
    "displace",
    () => ball.displace({ amplitude: 0.1, scale: 0.5, octaves: 3, seed: 7 }),
    {
      kind: "displace",
      shape: 0,
      amplitude: 0.1,
      scale: 0.5,
      octaves: 3,
      seed: 7,
    },
  ],
  ["elongate", () => ball.elongate([1, 0, 0], [0, 1, 0]), {
    kind: "elongate",
    shape: 0,
    lengths: [1, 0, 0],
    center: [0, 1, 0],
  }],
  ["mirror", () => ball.mirror("xz", [1, 0, 1]), {
    kind: "mirror",
    shape: 0,
    axes: "xz",
    center: [1, 0, 1],
  }],
  ["offset", () => ball.offset(0.1), {
    kind: "offset",
    shape: 0,
    distance: 0.1,
  }],
  ["orient", () => ball.orient([0, 1, 0], [1, 0, 0], [0, 0, 1]), {
    kind: "orient",
    shape: 0,
    from: [0, 1, 0],
    to: [1, 0, 0],
    pivot: [0, 0, 1],
  }],
  ["repeat", () => ball.repeat([1, 0, 0], [3, 1, 1]), {
    kind: "repeat",
    shape: 0,
    step: [1, 0, 0],
    count: [3, 1, 1],
  }],
  ["repeatPolar", () => ball.repeatPolar("y", 12, [0, 0, 1]), {
    kind: "repeatPolar",
    shape: 0,
    axis: "y",
    count: 12,
    center: [0, 0, 1],
  }],
  ["rotate", () => ball.rotate("z", 90, [1, 0, 0]), {
    kind: "rotate",
    shape: 0,
    axis: "z",
    degrees: 90,
    pivot: [1, 0, 0],
  }],
  ["scale", () => ball.scale([1, 2, 3], [0, 1, 0]), {
    kind: "scale",
    shape: 0,
    factor: [1, 2, 3],
    pivot: [0, 1, 0],
  }],
  ["shell", () => ball.shell(0.025), {
    kind: "shell",
    shape: 0,
    thickness: 0.025,
  }],
  ["translate", () => ball.translate([1, 2, 3]), {
    kind: "translate",
    shape: 0,
    offset: [1, 2, 3],
  }],
  ["twist", () => ball.twist("y", 90, [0, 1, 0]), {
    kind: "twist",
    shape: 0,
    axis: "y",
    degreesPerMeter: 90,
    center: [0, 1, 0],
  }],
];

for (const [call, shape, entry] of CASES) {
  Deno.test(`${call} writes its receiver and arguments`, () => {
    assert.deepStrictEqual(shapes3d(shape()), [BALL_ENTRY, entry]);
  });
}

Deno.test("scale by one factor writes the factor once per axis", () => {
  assert.deepStrictEqual(shapes3d(ball.scale(2)), [
    BALL_ENTRY,
    { kind: "scale", shape: 0, factor: [2, 2, 2] },
  ]);
});

Deno.test("a method leaves its receiver alone", () => {
  ball.translate([1, 0, 0]);
  assert.deepStrictEqual(shapes3d(ball), [BALL_ENTRY]);
});

Deno.test("a vector copies at the call", () => {
  const offset: [number, number, number] = [1, 2, 3];
  const shape = ball.translate(offset);
  offset[0] = 9;
  assert.deepStrictEqual(shapes3d(shape)[1], {
    kind: "translate",
    shape: 0,
    offset: [1, 2, 3],
  });
});
