import assert from "node:assert/strict";
import { int, json, type Material, material } from "./material.ts";
import { add } from "./step.ts";
import { ball, document } from "./test/tables.ts";

/** The `materials` table of a model that adds `ball` in `material`. */
function materials(material: Material) {
  return document([add("shape", ball, material)]).materials;
}

Deno.test("material writes each property in its form", () => {
  const chest = material({
    baseColor: "#8A5A2B",
    roughness: 0.8,
    lootTier: int(3),
    offsets: int([1, -2]),
    locked: true,
    tint: [0.5, 1],
    note: "oak",
    data: json({ drops: ["key", null], weight: 1.5 }),
  });
  assert.deepStrictEqual(materials(chest), [{
    kind: "material",
    properties: {
      baseColor: "#8A5A2B",
      roughness: 0.8,
      lootTier: { kind: "int", value: 3 },
      offsets: { kind: "int", value: [1, -2] },
      locked: true,
      tint: [0.5, 1],
      note: "oak",
      data: { kind: "json", value: { drops: ["key", null], weight: 1.5 } },
    },
  }]);
});

Deno.test("a property holding undefined stays out", () => {
  assert.deepStrictEqual(
    materials(material({ baseColor: "#FFFFFF", roughness: undefined })),
    [{ kind: "material", properties: { baseColor: "#FFFFFF" } }],
  );
});

Deno.test("json copies its value at the call", () => {
  const value = { count: 1 };
  const data = json(value);
  value.count = 2;
  assert.deepStrictEqual(
    materials(material({ baseColor: "#FFFFFF", data }))[0],
    {
      kind: "material",
      properties: {
        baseColor: "#FFFFFF",
        data: { kind: "json", value: { count: 1 } },
      },
    },
  );
});

Deno.test("a property the document cannot hold errors", () => {
  assert.throws(() => material({ baseColor: "#FFFFFF", roughness: NaN }), {
    message: "material roughness must be a finite number, not NaN",
  });
  assert.throws(() => material({ roughness: 0.5 } as never), {
    message: "material baseColor must be a string, not undefined",
  });
  assert.throws(() => material({ baseColor: "#FFFFFF", glow: null as never }), {
    message:
      "material glow must be a boolean, a number, a string, an array of numbers, an IntValue, or a JsonValue, not null",
  });
  assert.throws(() => int([1, -Infinity]), {
    message: "int value[1] must be a finite number, not -Infinity",
  });
});

Deno.test("json errors on a value JSON cannot hold", () => {
  assert.throws(() => json({ a: [1, NaN] }), {
    message: "json value.a[1] must be a finite number, not NaN",
  });
  assert.throws(() => json({ a: undefined }), {
    message: "json value.a must be a JSON value, not undefined",
  });
  assert.throws(() => json(new Date(0)), {
    message: "json value must be a JSON value, not a Date",
  });
  const loop: { self?: unknown } = {};
  loop.self = loop;
  assert.throws(() => json(loop), { message: "json value.self holds itself" });
});
