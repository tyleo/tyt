import assert from "node:assert/strict";
import * as check from "./check.ts";

Deno.test("an error shows the value it rejects", () => {
  const cases: [unknown, string][] = [
    [-0, "-0"],
    [10n, "10n"],
    ["x", '"x"'],
    [[1, [2, 3]], "[1, [2, 3]]"],
    [[1, 2, 3, 4, 5, 6, 7, 8, 9], "[1, 2, 3, 4, 5, 6, 7, 8, ...]"],
    [{ a: 1 }, "an object"],
    [new Map(), "a Map"],
    [() => 0, "a function"],
    [undefined, "undefined"],
  ];
  for (const [value, shown] of cases) {
    assert.throws(() => check.vec2("value", value), {
      message: `value must be a Vec2 of 2 finite numbers, not ${shown}`,
    });
  }
});

Deno.test("an options object holds only known keys", () => {
  assert.deepStrictEqual(check.knownOptions("call", undefined, ["a"]), {});
  assert.throws(() => check.knownOptions("call", [], ["a"]), {
    message: "call options must be an object, not []",
  });
});
