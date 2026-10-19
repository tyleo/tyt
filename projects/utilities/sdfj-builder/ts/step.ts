import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import { Material } from "./material.ts";
import { Pattern } from "./pattern.ts";
import { Shape3d } from "./shape3d.ts";
import type { Side, Vec3 } from "./types.ts";

/** One entry of a part's list of steps. */
export class Step {
  readonly #entry: Entry;

  /** The step whose `steps` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `steps` entry `step` writes. */
  static entry(step: Step): Entry {
    return step.#entry;
  }
}

/** The options of a `coat`. */
export interface CoatOptions {
  /** The sides the coat looks toward from each live cell. */
  sides?: Side[];

  /** The shape whose cells the coat stays inside. */
  within?: Shape3d;

  /** The `add` steps whose cells the coat stays on. */
  onto?: Step[];
}

/** The options of a `paint`. */
export interface PaintOptions {
  /** The `add` steps whose cells the paint stays on. */
  onto?: Step[];
}

/** Fills the cells of `shape` with `material`. */
export function add(
  name: string,
  shape: Shape3d,
  material: Material | Pattern,
): Step {
  const path = stepPath("add", name);
  return new Step({
    kind: "add",
    name,
    shape: check.instance(`${path} shape`, shape, Shape3d),
    ...stepMaterial(`${path} material`, material),
  });
}

/** Empties the cells of `shape`. */
export function carve(name: string, shape: Shape3d): Step {
  const path = stepPath("carve", name);
  return new Step({
    kind: "carve",
    name,
    shape: check.instance(`${path} shape`, shape, Shape3d),
  });
}

/** Recolors the live cells inside `shape` with `material`. */
export function paint(
  name: string,
  shape: Shape3d,
  material: Material | Pattern,
  options?: PaintOptions,
): Step {
  const path = stepPath("paint", name);
  const option = check.knownOptions(path, options, ["onto"]);
  return new Step({
    kind: "paint",
    name,
    shape: check.instance(`${path} shape`, shape, Shape3d),
    ...stepMaterial(`${path} material`, material),
    onto: check.optional(`${path} onto`, option.onto, onto),
  });
}

/**
 * Recolors the live cells whose point `depth` meters toward a side lies outside
 * the part's earlier shapes.
 */
export function coat(
  name: string,
  material: Material | Pattern,
  depth: number,
  options?: CoatOptions,
): Step {
  const path = stepPath("coat", name);
  const option = check.knownOptions(path, options, [
    "sides",
    "within",
    "onto",
  ]);
  return new Step({
    kind: "coat",
    name,
    ...stepMaterial(`${path} material`, material),
    depth: check.finite(`${path} depth`, depth),
    sides: check.optional(
      `${path} sides`,
      option.sides,
      (sidesPath, sides) =>
        check.list(sidesPath, sides, "an array of Sides", check.side),
    ),
    within: check.optional(
      `${path} within`,
      option.within,
      (withinPath, within) => check.instance(withinPath, within, Shape3d),
    ),
    onto: check.optional(`${path} onto`, option.onto, onto),
  });
}

/** Fills the cell holding each of `points` with `material`. */
export function set(
  name: string,
  points: Vec3 | Vec3[],
  material: Material | Pattern,
): Step {
  const path = stepPath("set", name);
  const single = Array.isArray(points) && typeof points[0] === "number";
  return new Step({
    kind: "set",
    name,
    points: single ? [check.vec3(`${path} points`, points)] : check.list(
      `${path} points`,
      points,
      "a Vec3 or an array of Vec3s",
      check.vec3,
    ),
    ...stepMaterial(`${path} material`, material),
  });
}

/** The path `call`'s errors start with, after checking `name`. */
function stepPath(call: string, name: unknown): string {
  return `${call} ${JSON.stringify(check.text(`${call} name`, name))}`;
}

/** The one or more `add` steps the array `value` holds. */
function onto(path: string, value: unknown): Step[] {
  const steps = check.list(
    path,
    value,
    "an array of add Steps",
    (itemPath, item) =>
      item instanceof Step
        ? addStep(itemPath, item)
        : check.fail(itemPath, "an add Step", item),
  );
  return steps.length > 0
    ? steps
    : check.fail(path, "an array of at least one add Step", value);
}

/** `step` after checking that it is an `add`. */
function addStep(path: string, step: Step): Step {
  const { kind, name } = Step.entry(step);
  if (kind !== "add") {
    throw new Error(
      `${path} must be an add Step, not the ${kind} ${JSON.stringify(name)}`,
    );
  }
  return step;
}

/** The key and value a step writes for `value`. */
function stepMaterial(
  path: string,
  value: unknown,
): { material: Material } | { pattern: Pattern } {
  if (value instanceof Material) {
    return { material: value };
  }
  if (value instanceof Pattern) {
    return { pattern: value };
  }
  return check.fail(path, "a Material or a Pattern", value);
}
