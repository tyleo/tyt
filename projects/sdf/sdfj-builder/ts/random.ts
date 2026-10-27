import * as check from "./check.ts";

/** A seeded generator. Each draw advances it. */
export class Random {
  #state: number;

  /** The generator starting from the 32-bit `state`. */
  constructor(state: number) {
    this.#state = state;
  }

  /** A number from `min` up to `max`. */
  range(min: number, max: number): number {
    check.finite("random range min", min);
    if (check.finite("random range max", max) < min) {
      check.fail("random range max", `at least min ${min}`, max);
    }
    return min + (max - min) * this.#next();
  }

  /** One of `items`. */
  pick<T>(items: readonly T[]): T {
    if (!Array.isArray(items) || items.length === 0) {
      check.fail("random pick items", "an array of at least one item", items);
    }
    return items[Math.floor(items.length * this.#next())];
  }

  /** A number from 0 up to 1, by mulberry32. */
  #next(): number {
    this.#state = (this.#state + 0x6d2b79f5) | 0;
    let mixed = Math.imul(this.#state ^ (this.#state >>> 15), this.#state | 1);
    mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
    return ((mixed ^ (mixed >>> 14)) >>> 0) / 2 ** 32;
  }
}

/** A generator whose draws repeat on every build from the same `seed`. */
export function random(seed: number): Random {
  // Both halves of the seed's bits feed the state. `+ 0` turns -0 into 0.
  const bits = new DataView(new ArrayBuffer(8));
  bits.setFloat64(0, check.finite("random seed", seed) + 0);
  return new Random(
    Math.imul(bits.getUint32(0), 0x9e3779b1) ^ bits.getUint32(4),
  );
}
