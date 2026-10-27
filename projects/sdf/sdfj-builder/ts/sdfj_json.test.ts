import assert from "node:assert/strict";
import { sdfjDocument } from "./sdfj_document.ts";
import { sdfjDocumentFromJson, sdfjJson } from "./sdfj_json.ts";
import { add } from "./step.ts";
import { ball, white } from "./test/tables.ts";

Deno.test("sdfjJson leaves out each empty table and names map", () => {
  assert.deepStrictEqual(sdfjJson(sdfjDocument("model", [])), {
    version: 1,
    nodes: [{ name: "model", childObjects: [], childNodes: [] }],
    rootNodes: [0],
  });
  const named = sdfjDocument("model", [], [["white", white]]);
  assert.deepStrictEqual(sdfjJson(named).names, { materials: { white: 0 } });
});

Deno.test("sdfjDocumentFromJson reads a missing table as empty", () => {
  const document = sdfjDocument("model", [add("ball", ball, white)], [
    ["white", white],
  ]);
  assert.deepStrictEqual(sdfjDocumentFromJson(sdfjJson(document)), document);
  assert.deepStrictEqual(
    sdfjDocumentFromJson({ version: 1 }),
    sdfjDocumentFromJson({
      version: 1,
      shapes3d: [],
      names: { materials: {} },
    }),
  );
});
