import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import type { Vec2 } from "./types.ts";

/**
 * A 2D region in a plane with axes u and v. A method returns a new shape and
 * leaves its receiver alone.
 */
export class Shape2d {
  readonly #entry: Entry;

  /** The shape whose `shapes2d` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `shapes2d` entry `shape` writes. */
  static entry(shape: Shape2d): Entry {
    return shape.#entry;
  }

  /** Moves the shape by `offset`. */
  translate(offset: Vec2): Shape2d {
    return new Shape2d({
      kind: "translate",
      shape: this,
      offset: check.vec2("translate offset", offset),
    });
  }

  /** Turns the shape `degrees` from +u toward +v about `pivot`. */
  rotate(degrees: number, pivot?: Vec2): Shape2d {
    return new Shape2d({
      kind: "rotate",
      shape: this,
      degrees: check.finite("rotate degrees", degrees),
      pivot: check.optional("rotate pivot", pivot, check.vec2),
    });
  }

  /**
   * Scales the shape by `factor` about `pivot`. A negative factor reflects its
   * axis.
   */
  scale(factor: number | Vec2, pivot?: Vec2): Shape2d {
    return new Shape2d({
      kind: "scale",
      shape: this,
      factor: check.numberOrVec2("scale factor", factor),
      pivot: check.optional("scale pivot", pivot, check.vec2),
    });
  }

  /** Adds the shape's reflection across each of `axes` through `center`. */
  mirror(axes: "u" | "v" | "uv", center?: Vec2): Shape2d {
    return new Shape2d({
      kind: "mirror",
      shape: this,
      axes: check.axes2d("mirror axes", axes),
      center: check.optional("mirror center", center, check.vec2),
    });
  }

  /** Copies the shape `count` times along each axis, `step` apart. */
  repeat(step: Vec2, count: Vec2): Shape2d {
    return new Shape2d({
      kind: "repeat",
      shape: this,
      step: check.vec2("repeat step", step),
      count: check.vec2("repeat count", count),
    });
  }

  /** Copies the shape `count` times evenly around `center`. */
  repeatPolar(count: number, center?: Vec2): Shape2d {
    return new Shape2d({
      kind: "repeatPolar",
      shape: this,
      count: check.finite("repeatPolar count", count),
      center: check.optional("repeatPolar center", center, check.vec2),
    });
  }

  /** Grows the shape by `distance`. */
  offset(distance: number): Shape2d {
    return new Shape2d({
      kind: "offset",
      shape: this,
      distance: check.finite("offset distance", distance),
    });
  }

  /** Keeps the shape's outer `thickness` and hollows the rest. */
  shell(thickness: number): Shape2d {
    return new Shape2d({
      kind: "shell",
      shape: this,
      thickness: check.finite("shell thickness", thickness),
    });
  }
}
