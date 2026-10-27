import assert from "node:assert/strict";
import { helix } from "./paths.ts";

Deno.test("helix climbs its height over its turns", () => {
  assert.deepStrictEqual(helix([1, 2, 3], 2, 1, 1, { perTurn: 4 }), [
    [1, 2, 5],
    [3, 2.25, 3],
    [1, 2.5, 1],
    [-1, 2.75, 3],
    [1, 3, 5],
  ]);
});

Deno.test("helix starts on the next axis around and winds by turns", () => {
  assert.deepStrictEqual(
    helix([0, 0, 0], 1, 2, -0.5, { axis: "z", start: 90, perTurn: 4 }),
    [[0, 1, 0], [1, 0, 1], [0, -1, 2]],
  );
});

Deno.test("helix takes a whole number of points", () => {
  assert.equal(helix([0, 0, 0], 1, 3, 1.25).length, 16);
  assert.throws(() => helix([0, 0, 0], 1, 3, 1.3), {
    message:
      "helix turns must be a whole number of points at 12 per turn, not 1.3",
  });
  assert.throws(() => helix([0, 0, 0], 0, 3, 1), {
    message: "helix radius must be above zero, not 0",
  });
  assert.throws(() => helix([0, 0, 0], 1, 3, 0), {
    message: "helix turns must be a nonzero number, not 0",
  });
});
