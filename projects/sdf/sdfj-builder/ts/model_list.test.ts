import assert from "node:assert/strict";
import { modelList, namedExports } from "./model_list.ts";
import { shades } from "./shades.ts";
import { add } from "./step.ts";
import { ball, white } from "./test/tables.ts";

Deno.test("the default export lists the model's steps and parts", () => {
  const step = add("ball", ball, white);
  assert.deepStrictEqual(modelList({ default: [step] }), [step]);
});

Deno.test("a default export the builder cannot write errors", () => {
  assert.throws(() => modelList({}), {
    message: "the model has no default export",
  });
  assert.throws(() => modelList({ default: add("ball", ball, white) }), {
    message:
      "the default export must be an array of Steps and Parts, not a Step",
  });
  assert.throws(() => modelList({ default: [ball] }), {
    message: "the default export[0] must be a Step or a Part, not a Shape3d",
  });
});

Deno.test("the named exports leave out the default and functions", () => {
  const model = { ball, default: [], leg: () => ball, white };
  assert.deepStrictEqual(namedExports(model), [
    ["ball", ball],
    ["white", white],
  ]);
});

Deno.test("a named export holding no entry errors", () => {
  assert.throws(() => namedExports({ shades: shades(white) }), {
    message:
      "the export shades must be a Material, a Part, a Pattern, a Shape2d, a Shape3d, a Step, or a function, not [a Material, a Material, a Material]",
  });
  assert.throws(() => namedExports({ size: 0.025 }), {
    message:
      "the export size must be a Material, a Part, a Pattern, a Shape2d, a Shape3d, a Step, or a function, not 0.025",
  });
});
