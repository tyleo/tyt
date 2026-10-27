import assert from "node:assert/strict";
import * as api from "./api.ts";
import { Font } from "./font.ts";
import { putApiInScope } from "./put_api_in_scope.ts";

Deno.test("putApiInScope puts every API name on globalThis", () => {
  const sign = { kind: "installed", name: "sign", font: "Futura" };
  putApiInScope([], [sign], { legs: 4 });
  for (const [name, value] of Object.entries(api)) {
    assert.equal(Reflect.get(globalThis, name), value, name);
  }
  assert.equal(globalThis.vars.legs, 4);
  assert.deepEqual(Font.entry(globalThis.fonts.sign), sign);
  assert.throws(() => globalThis.mat.oak, {
    message: 'mat has no material "oak" because the build reads no library',
  });
  assert.throws(() => globalThis.lib.parts.stool, {
    message: 'lib.parts has no part "stool" because the build reads no library',
  });
});
