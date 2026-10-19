import * as check from "./check.ts";
import { Shape2d } from "./shape2d.ts";
import { Shape3d } from "./shape3d.ts";
import type { Axis, Vec2, Vec3 } from "./types.ts";

/** The solid `profile` sweeps from `from` to `to` along `axis`. */
export function extrude(
  profile: Shape2d,
  options: { axis?: Axis; from: number; to: number },
): Shape3d {
  const option = check.knownOptions("extrude", options, ["axis", "from", "to"]);
  return new Shape3d({
    kind: "extrude",
    profile: check.instance("extrude profile", profile, Shape2d),
    axis: check.optional("extrude axis", option.axis, check.axis),
    from: check.finite("extrude from", option.from),
    to: check.finite("extrude to", option.to),
  });
}

/** The solid `profile` sweeps around `axis` through `center`. */
export function revolve(
  profile: Shape2d,
  options?: { axis?: Axis; center?: Vec3 },
): Shape3d {
  const option = check.knownOptions("revolve", options, ["axis", "center"]);
  return new Shape3d({
    kind: "revolve",
    profile: check.instance("revolve profile", profile, Shape2d),
    axis: check.optional("revolve axis", option.axis, check.axis),
    center: check.optional("revolve center", option.center, check.vec3),
  });
}

/** The solid the outline of `[radius, height]` points sweeps around `axis`. */
export function lathe(
  points: Vec2[],
  options?: { axis?: Axis; center?: Vec3 },
): Shape3d {
  const option = check.knownOptions("lathe", options, ["axis", "center"]);
  return new Shape3d({
    kind: "lathe",
    points: check.list("lathe points", points, "an array of Vec2s", check.vec2),
    axis: check.optional("lathe axis", option.axis, check.axis),
    center: check.optional("lathe center", option.center, check.vec3),
  });
}

/** The solid `profile` traces along the smooth curve through `points`. */
export function sweep(
  profile: Shape2d,
  points: Vec3[],
  options?: { scales?: number[]; up?: Vec3 },
): Shape3d {
  const option = check.knownOptions("sweep", options, ["scales", "up"]);
  return new Shape3d({
    kind: "sweep",
    profile: check.instance("sweep profile", profile, Shape2d),
    points: check.list("sweep points", points, "an array of Vec3s", check.vec3),
    scales: check.optional(
      "sweep scales",
      option.scales,
      (path, value) =>
        check.list(path, value, "an array of numbers", check.finite),
    ),
    up: check.optional("sweep up", option.up, check.vec3),
  });
}
