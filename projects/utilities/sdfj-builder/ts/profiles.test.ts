import assert from "node:assert/strict";
import { sphere } from "./primitives.ts";
import { circle } from "./primitives2d.ts";
import { extrude, lathe, revolve, sweep } from "./profiles.ts";
import { add } from "./step.ts";
import { document, white } from "./test/tables.ts";

Deno.test("extrude writes its profile and its range", () => {
  const shape = extrude(circle([0, 0], 1), { axis: "x", from: 0, to: 1 });
  const { shapes2d, shapes3d } = document([add("shape", shape, white)]);
  assert.deepStrictEqual(shapes2d, [
    { kind: "circle", center: [0, 0], radius: 1 },
  ]);
  assert.deepStrictEqual(shapes3d, [
    { kind: "extrude", profile: 0, axis: "x", from: 0, to: 1 },
  ]);
});

Deno.test("revolve writes its profile, axis, and center", () => {
  const shape = revolve(circle([1, 0], 0.5), { axis: "z", center: [0, 1, 0] });
  const { shapes2d, shapes3d } = document([add("shape", shape, white)]);
  assert.deepStrictEqual(shapes2d, [
    { kind: "circle", center: [1, 0], radius: 0.5 },
  ]);
  assert.deepStrictEqual(shapes3d, [
    { kind: "revolve", profile: 0, axis: "z", center: [0, 1, 0] },
  ]);
});

Deno.test("lathe writes its points, axis, and center", () => {
  const shape = lathe([[0.5, 0], [0.25, 1]], { axis: "x", center: [0, 0, 1] });
  const { shapes2d, shapes3d } = document([add("shape", shape, white)]);
  assert.deepStrictEqual(shapes2d, []);
  assert.deepStrictEqual(shapes3d, [
    {
      kind: "lathe",
      points: [[0.5, 0], [0.25, 1]],
      axis: "x",
      center: [0, 0, 1],
    },
  ]);
});

Deno.test("sweep writes its profile, points, scales, and up", () => {
  const shape = sweep(circle([0, 0], 0.1), [[0, 0, 0], [1, 1, 0]], {
    scales: [1, 0.5],
    up: [0, 0, 1],
  });
  const { shapes2d, shapes3d } = document([add("shape", shape, white)]);
  assert.deepStrictEqual(shapes2d, [
    { kind: "circle", center: [0, 0], radius: 0.1 },
  ]);
  assert.deepStrictEqual(shapes3d, [
    {
      kind: "sweep",
      profile: 0,
      points: [[0, 0, 0], [1, 1, 0]],
      scales: [1, 0.5],
      up: [0, 0, 1],
    },
  ]);
});

Deno.test("a profile has to be a 2D shape", () => {
  assert.throws(
    () => extrude(sphere([0, 0, 0], 1) as never, { from: 0, to: 1 }),
    { message: "extrude profile must be a Shape2d, not a Shape3d" },
  );
});
