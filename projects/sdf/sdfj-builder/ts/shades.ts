import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import { Material } from "./material.ts";

/** One `shades` call, which its shade materials reference. */
export class Shades {
  readonly #entry: Entry;

  /** The call whose `shades` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `shades` entry `shades` writes. */
  static entry(shades: Shades): Entry {
    return shades.#entry;
  }
}

/**
 * `count` versions of `base` from darkest to lightest, with neighbors `spread`
 * apart in perceived lightness.
 */
export function shades(
  base: Material,
  options?: { count?: number; spread?: number },
): Material[] {
  const option = check.knownOptions("shades", options, ["count", "spread"]);
  const baseMaterial = check.instance("shades base", base, Material);
  const count = check.count(
    "shades count",
    option.count === undefined ? 3 : option.count,
  );
  const call = new Shades({
    base: baseMaterial,
    count,
    spread: check.optional("shades spread", option.spread, check.finite),
  });
  return Array.from(
    { length: count },
    (_, index) => new Material({ kind: "shade", shades: call, index }),
  );
}
