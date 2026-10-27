import assert from "node:assert/strict";
import { bands } from "./pattern.ts";
import { shades } from "./shades.ts";
import { add } from "./step.ts";
import { ball, document, white } from "./test/tables.ts";

Deno.test("shades writes its call and one entry per shade", () => {
  const pattern = bands(shades(white, { count: 2, spread: 0.1 }), 0.05, {
    axis: "x",
  });
  const { materials, shades: calls } = document([add("shape", ball, pattern)]);
  assert.deepStrictEqual(materials, [
    { kind: "material", properties: { baseColor: "#FFFFFF" } },
    { kind: "shade", shades: 0, index: 0 },
    { kind: "shade", shades: 0, index: 1 },
  ]);
  assert.deepStrictEqual(calls, [{ base: 0, count: 2, spread: 0.1 }]);
});

Deno.test("shades writes its count when the model leaves it out", () => {
  const middle = shades(white)[1];
  const { materials, shades: calls } = document([add("shape", ball, middle)]);
  assert.deepStrictEqual(materials, [
    { kind: "material", properties: { baseColor: "#FFFFFF" } },
    { kind: "shade", shades: 0, index: 1 },
  ]);
  assert.deepStrictEqual(calls, [{ base: 0, count: 3 }]);
});

Deno.test("shades needs a whole count", () => {
  assert.throws(() => shades(white, { count: 2.5 }), {
    message: "shades count must be a whole number above zero, not 2.5",
  });
});
