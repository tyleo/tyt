import assert from "node:assert/strict";
import { varsGlobal } from "./vars.ts";

Deno.test("vars reads each variable the build sets", () => {
  const vars = varsGlobal({ legs: 4, wood: "oak", size: [1, 2] });
  assert.equal(vars.legs, 4);
  assert.equal(vars.wood, "oak");
  assert.deepEqual(vars.size, [1, 2]);
});

Deno.test("reading a variable the build lacks errors", () => {
  assert.throws(() => varsGlobal({}).legs, {
    message: 'vars has no variable "legs" because the build sets none',
  });
  assert.throws(() => varsGlobal({ height: 1, width: 2 }).legs, {
    message:
      'vars has no variable "legs" because the build sets only "height", "width"',
  });
});
