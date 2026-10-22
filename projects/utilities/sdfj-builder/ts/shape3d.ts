import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import type { Axes, Axis, Side, Vec3 } from "./types.ts";

/** A 3D region. A method returns a new shape and leaves its receiver alone. */
export class Shape3d {
  readonly #entry: Entry;

  /** The shape whose `shapes3d` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `shapes3d` entry `shape` writes. */
  static entry(shape: Shape3d): Entry {
    return shape.#entry;
  }

  /** Moves the shape by `offset`. */
  translate(offset: Vec3): Shape3d {
    return new Shape3d({
      kind: "translate",
      shape: this,
      offset: check.vec3("translate offset", offset),
    });
  }

  /** Turns the shape `degrees` about `axis` through `pivot`. */
  rotate(axis: Axis, degrees: number, pivot?: Vec3): Shape3d {
    return new Shape3d({
      kind: "rotate",
      shape: this,
      axis: check.axis("rotate axis", axis),
      degrees: check.finite("rotate degrees", degrees),
      pivot: check.optional("rotate pivot", pivot, check.vec3),
    });
  }

  /** Turns the shape about `pivot` until `from` points along `to`. */
  orient(from: Vec3, to: Vec3, pivot?: Vec3): Shape3d {
    return new Shape3d({
      kind: "orient",
      shape: this,
      from: check.vec3("orient from", from),
      to: check.vec3("orient to", to),
      pivot: check.optional("orient pivot", pivot, check.vec3),
    });
  }

  /**
   * Scales the shape by `factor` about `pivot`. A negative factor reflects its
   * axis.
   */
  scale(factor: number | Vec3, pivot?: Vec3): Shape3d {
    return new Shape3d({
      kind: "scale",
      shape: this,
      factor: check.numberOrVec3("scale factor", factor),
      pivot: check.optional("scale pivot", pivot, check.vec3),
    });
  }

  /** Adds the shape's reflection across each of `axes` through `center`. */
  mirror(axes: Axes, center?: Vec3): Shape3d {
    return new Shape3d({
      kind: "mirror",
      shape: this,
      axes: check.axes("mirror axes", axes),
      center: check.optional("mirror center", center, check.vec3),
    });
  }

  /** Copies the shape `count` times along each axis, `step` apart. */
  repeat(step: Vec3, count: Vec3): Shape3d {
    return new Shape3d({
      kind: "repeat",
      shape: this,
      step: check.vec3("repeat step", step),
      count: check.vec3("repeat count", count),
    });
  }

  /** Copies the shape `count` times evenly around `axis` through `center`. */
  repeatPolar(axis: Axis, count: number, center?: Vec3): Shape3d {
    return new Shape3d({
      kind: "repeatPolar",
      shape: this,
      axis: check.axis("repeatPolar axis", axis),
      count: check.finite("repeatPolar count", count),
      center: check.optional("repeatPolar center", center, check.vec3),
    });
  }

  /** Grows the shape by `distance`. */
  offset(distance: number): Shape3d {
    return new Shape3d({
      kind: "offset",
      shape: this,
      distance: check.finite("offset distance", distance),
    });
  }

  /** Keeps the shape's outer `thickness` and hollows the rest. */
  shell(thickness: number): Shape3d {
    return new Shape3d({
      kind: "shell",
      shape: this,
      thickness: check.finite("shell thickness", thickness),
    });
  }

  /** Moves the shape's halves apart across `center` by `lengths`. */
  elongate(lengths: Vec3, center?: Vec3): Shape3d {
    return new Shape3d({
      kind: "elongate",
      shape: this,
      lengths: check.vec3("elongate lengths", lengths),
      center: check.optional("elongate center", center, check.vec3),
    });
  }

  /** Twists the shape about `axis` by `degreesPerMeter`. */
  twist(axis: Axis, degreesPerMeter: number, center?: Vec3): Shape3d {
    return new Shape3d({
      kind: "twist",
      shape: this,
      axis: check.axis("twist axis", axis),
      degreesPerMeter: check.finite("twist degreesPerMeter", degreesPerMeter),
      center: check.optional("twist center", center, check.vec3),
    });
  }

  /** Curls the shape's `along` axis into an arc of `radius` toward `toward`. */
  bend(along: Axis, toward: Side, radius: number, pivot?: Vec3): Shape3d {
    return new Shape3d({
      kind: "bend",
      shape: this,
      along: check.axis("bend along", along),
      toward: check.side("bend toward", toward),
      radius: check.finite("bend radius", radius),
      pivot: check.optional("bend pivot", pivot, check.vec3),
    });
  }

  /** Roughens the surface with fractal noise up to `amplitude` deep. */
  displace(
    options: {
      amplitude: number;
      scale: number;
      octaves?: number;
      seed: number;
    },
  ): Shape3d {
    const option = check.knownOptions("displace", options, [
      "amplitude",
      "scale",
      "octaves",
      "seed",
    ]);
    return new Shape3d({
      kind: "displace",
      shape: this,
      amplitude: check.finite("displace amplitude", option.amplitude),
      scale: check.finite("displace scale", option.scale),
      octaves: check.optional("displace octaves", option.octaves, check.finite),
      seed: check.finite("displace seed", option.seed),
    });
  }
}
