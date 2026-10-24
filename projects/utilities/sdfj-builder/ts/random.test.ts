import assert from "node:assert/strict";
import { random } from "./random.ts";

const sequence = (seed: number) => {
  const generator = random(seed);
  return Array.from({ length: 4 }, () => generator.range(0, 1));
};

Deno.test("random draws the same sequence from one seed", () => {
  assert.deepStrictEqual(sequence(7), sequence(7));
  assert.equal(new Set(sequence(7)).size, 4);
});

Deno.test("random draws differ between seeds", () => {
  const firsts = [0, 1, 2, 0.5, 0.25, -1].map((seed) => sequence(seed)[0]);
  assert.equal(new Set(firsts).size, firsts.length);
  assert.deepStrictEqual(sequence(-0), sequence(0));
});

Deno.test("random range draws from min up to max", () => {
  const generator = random(3);
  for (let draw = 0; draw < 1000; draw++) {
    const value = generator.range(0.2, 0.3);
    assert.ok(value >= 0.2 && value < 0.3, `${value}`);
  }
  assert.equal(generator.range(0.4, 0.4), 0.4);
  assert.throws(() => generator.range(0.3, 0.2), {
    message: "random range max must be at least min 0.3, not 0.2",
  });
  assert.throws(() => generator.range(NaN, 1), {
    message: "random range min must be a finite number, not NaN",
  });
});

Deno.test("random pick draws every item of the list", () => {
  const generator = random(5);
  const picked = new Set(
    Array.from({ length: 100 }, () => generator.pick(["a", "b", "c"])),
  );
  assert.deepStrictEqual([...picked].sort(), ["a", "b", "c"]);
  assert.throws(() => generator.pick([]), {
    message: "random pick items must be an array of at least one item, not []",
  });
});

Deno.test("random takes a finite seed", () => {
  assert.throws(() => random(Infinity), {
    message: "random seed must be a finite number, not Infinity",
  });
});
