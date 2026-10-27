import assert from "node:assert/strict";
import { jsonText } from "./json_text.ts";

Deno.test("jsonText writes compact JSON in key order", () => {
  assert.equal(
    jsonText({ b: [1, 0.1, 1e21], a: { s: "x\n", t: true, n: null } }),
    '{"b":[1,0.1,1e+21],"a":{"s":"x\\n","t":true,"n":null}}',
  );
});

Deno.test("jsonText keeps the sign of -0", () => {
  assert.equal(jsonText([-0, 0]), "[-0,0]");
});

Deno.test("jsonText errors on a value without a JSON form", () => {
  assert.throws(() => jsonText([NaN]), {
    message: "a document number must be a finite number, not NaN",
  });
  assert.throws(() => jsonText({ a: undefined }), {
    message: "a document value must be JSON, not undefined",
  });
});
