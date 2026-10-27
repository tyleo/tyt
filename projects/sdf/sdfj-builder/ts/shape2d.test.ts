import assert from "node:assert/strict";
import type { Entry } from "./entry.ts";
import type { Shape2d } from "./shape2d.ts";
import { disk, DISK_ENTRY, shapes2d } from "./test/tables.ts";

const CASES: [string, () => Shape2d, Entry][] = [
  ["mirror", () => disk.mirror("uv", [1, 0]), {
    kind: "mirror",
    shape: 0,
    axes: "uv",
    center: [1, 0],
  }],
  ["offset", () => disk.offset(0.1), {
    kind: "offset",
    shape: 0,
    distance: 0.1,
  }],
  ["repeat", () => disk.repeat([1, 0], [3, 1]), {
    kind: "repeat",
    shape: 0,
    step: [1, 0],
    count: [3, 1],
  }],
  ["repeatPolar", () => disk.repeatPolar(6, [0, 1]), {
    kind: "repeatPolar",
    shape: 0,
    count: 6,
    center: [0, 1],
  }],
  ["rotate", () => disk.rotate(45, [1, 0]), {
    kind: "rotate",
    shape: 0,
    degrees: 45,
    pivot: [1, 0],
  }],
  ["scale", () => disk.scale([1, 2], [0, 1]), {
    kind: "scale",
    shape: 0,
    factor: [1, 2],
    pivot: [0, 1],
  }],
  ["shell", () => disk.shell(0.1), {
    kind: "shell",
    shape: 0,
    thickness: 0.1,
  }],
  ["translate", () => disk.translate([1, 2]), {
    kind: "translate",
    shape: 0,
    offset: [1, 2],
  }],
];

for (const [call, shape, entry] of CASES) {
  Deno.test(`2D ${call} writes its receiver and arguments`, () => {
    assert.deepStrictEqual(shapes2d(shape()), [DISK_ENTRY, entry]);
  });
}

Deno.test("2D scale by one factor writes the factor once per axis", () => {
  assert.deepStrictEqual(shapes2d(disk.scale(2)), [
    DISK_ENTRY,
    { kind: "scale", shape: 0, factor: [2, 2] },
  ]);
});

Deno.test("2D mirror takes only the plane's axes", () => {
  assert.throws(() => disk.mirror("x" as never), {
    message: 'mirror axes must be one of "u", "v", "uv", not "x"',
  });
});
