import * as check from "./check.ts";
import { Step } from "./step.ts";
import type { AxisRotation, Vec3 } from "./types.ts";

/** The fields a part holds before the builder writes its node and object. */
export interface PartEntry {
  /** The name of the part's node and object. */
  readonly name: string;

  /** The joint the part turns about. */
  readonly pivot?: Vec3;

  /** The move of the part's steps, child parts, and pivot. */
  readonly offset?: Vec3;

  /** The rotations of the part about its pivot, in order. */
  readonly rotation?: readonly AxisRotation[];

  /** Whether the part keeps its node when the hierarchy flattens. */
  readonly joint?: boolean;

  /** The part's steps and child parts in list order. */
  readonly list: readonly (Step | Part)[];
}

/** A group of steps that voxelizes into an object turning about a joint. */
export class Part {
  readonly #entry: PartEntry;

  /** The part holding `entry`. */
  constructor(entry: PartEntry) {
    this.#entry = entry;
    Object.freeze(this);
  }

  /** The fields `part` holds. */
  static entry(part: Part): PartEntry {
    return part.#entry;
  }
}

/** The part `name` that voxelizes `list` and turns about `pivot`. */
export function part(
  name: string,
  options: {
    pivot?: Vec3;
    offset?: Vec3;
    rotation?: AxisRotation[];
    joint?: boolean;
  },
  list: (Step | Part)[],
): Part {
  const path = `part ${JSON.stringify(check.text("part name", name))}`;
  const option = check.knownOptions(path, options, [
    "pivot",
    "offset",
    "rotation",
    "joint",
  ]);
  return new Part({
    name,
    pivot: check.optional(`${path} pivot`, option.pivot, check.vec3),
    offset: check.optional(`${path} offset`, option.offset, check.vec3),
    rotation: check.optional(
      `${path} rotation`,
      option.rotation,
      (rotationPath, value) =>
        check.list(
          rotationPath,
          value,
          "an array of AxisRotations",
          check.axisRotation,
        ),
    ),
    joint: check.optional(`${path} joint`, option.joint, check.boolean),
    list: stepsAndParts(`${path} list`, list),
  });
}

/** A copy of the array `value` of steps and parts. */
export function stepsAndParts(path: string, value: unknown): (Step | Part)[] {
  return check.list(
    path,
    value,
    "an array of Steps and Parts",
    (itemPath, item) =>
      item instanceof Step || item instanceof Part
        ? item
        : check.fail(itemPath, "a Step or a Part", item),
  );
}
