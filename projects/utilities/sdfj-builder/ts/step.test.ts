import assert from "node:assert/strict";
import type { Entry } from "./entry.ts";
import { checker } from "./pattern.ts";
import { add, carve, coat, paint, set, type Step } from "./step.ts";
import { ball, disk, document, white } from "./test/tables.ts";

const CASES: [string, () => Step, Entry][] = [
  ["add", () => add("seat", ball, white), {
    kind: "add",
    name: "seat",
    shape: 0,
    material: 0,
  }],
  ["carve", () => carve("doorway", ball), {
    kind: "carve",
    name: "doorway",
    shape: 0,
  }],
  [
    "coat",
    () =>
      coat("snow", white, 0.05, {
        sides: ["+y", "-x"],
        slope: 40,
        within: ball,
      }),
    {
      kind: "coat",
      name: "snow",
      material: 0,
      sides: ["+y", "-x"],
      depth: 0.05,
      slope: 40,
      within: 0,
    },
  ],

  ["paint", () => paint("gilt", ball, checker([white], 0.05)), {
    kind: "paint",
    name: "gilt",
    shape: 0,
    pattern: 0,
  }],
  ["set", () => set("eyes", [[0, 0, 0], [1, 1, 1]], 0.05, white), {
    kind: "set",
    name: "eyes",
    points: [[0, 0, 0], [1, 1, 1]],
    size: 0.05,
    material: 0,
  }],
];

for (const [call, step, entry] of CASES) {
  Deno.test(`${call} writes its name and arguments`, () => {
    assert.deepStrictEqual(document([step()]).steps, [entry]);
  });
}

Deno.test("set of one point writes a list of one point", () => {
  assert.deepStrictEqual(document([set("eye", [1, 2, 3], 0.05, white)]).steps, [
    { kind: "set", name: "eye", points: [[1, 2, 3]], size: 0.05, material: 0 },
  ]);
});

Deno.test("onto writes the indices of earlier add steps", () => {
  const body = add("body", ball, white);
  const rim = add("rim", ball, white);
  assert.deepStrictEqual(
    document([
      body,
      rim,
      paint("gilt", ball, white, { onto: [body] }),
      coat("snow", white, 0.05, { onto: [body, rim] }),
    ]).steps.slice(2),
    [
      { kind: "paint", name: "gilt", shape: 0, material: 0, onto: [0] },
      { kind: "coat", name: "snow", material: 0, depth: 0.05, onto: [0, 1] },
    ],
  );
});

Deno.test("onto lists add steps earlier in the list", () => {
  const body = add("body", ball, white);
  assert.throws(() => paint("gilt", ball, white, { onto: [] }), {
    message: 'paint "gilt" onto must be an array of at least one add Step, ' +
      "not []",
  });
  assert.throws(
    () => coat("snow", white, 0.05, { onto: [carve("cut", ball)] }),
    { message: 'coat "snow" onto[0] must be an add Step, not the carve "cut"' },
  );
  assert.throws(
    () => document([paint("gilt", ball, white, { onto: [body] }), body]),
    {
      message: 'paint "gilt" onto must list steps earlier in its list, ' +
        'not "body"',
    },
  );
});

Deno.test("a step's arguments have to fit its entry", () => {
  assert.throws(() => add("seat", disk as never, white), {
    message: 'add "seat" shape must be a Shape3d, not a Shape2d',
  });
  assert.throws(() => paint("gilt", ball, "gold" as never), {
    message:
      'paint "gilt" material must be a Material or a Pattern, not "gold"',
  });
  assert.throws(() => coat("snow", white, 0.05, { side: ["+y"] } as never), {
    message: 'coat "snow" takes no option "side"',
  });
  assert.throws(() => carve(3 as never, ball), {
    message: "carve name must be a string, not 3",
  });
});
