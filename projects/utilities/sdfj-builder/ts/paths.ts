import * as check from "./check.ts";
import type { Axis, Vec3 } from "./types.ts";

/** The points of a coil climbing `height` over `turns` turns from `base`. */
export function helix(
  base: Vec3,
  radius: number,
  height: number,
  turns: number,
  options?: { axis?: Axis; start?: number; perTurn?: number },
): Vec3[] {
  const option = check.knownOptions("helix", options, [
    "axis",
    "start",
    "perTurn",
  ]);
  const [x, y, z] = check.vec3("helix base", base);
  if (check.finite("helix radius", radius) <= 0) {
    check.fail("helix radius", "above zero", radius);
  }
  check.finite("helix height", height);
  if (check.finite("helix turns", turns) === 0) {
    check.fail("helix turns", "a nonzero number", turns);
  }
  const axis = check.optional("helix axis", option.axis, check.axis) ?? "y";
  const start = check.optional("helix start", option.start, check.finite) ?? 0;
  const perTurn =
    check.optional("helix perTurn", option.perTurn, check.count) ?? 12;
  const steps = Math.abs(turns) * perTurn;
  if (!Number.isInteger(steps)) {
    check.fail(
      "helix turns",
      `a whole number of points at ${perTurn} per turn`,
      turns,
    );
  }

  // The coil starts on the next axis around and turns toward the one after,
  // the way a positive rotate turns.
  const along = ["x", "y", "z"].indexOf(axis);
  const across = (along + 1) % 3;
  const toward = (along + 2) % 3;
  const points: Vec3[] = [];
  for (let step = 0; step <= steps; step++) {
    const [cos, sin] = turn(start + Math.sign(turns) * 360 * step / perTurn);
    const point: Vec3 = [x, y, z];
    point[along] += height * step / steps;
    point[across] += radius * cos;
    point[toward] += radius * sin;
    points.push(point);
  }
  return points;
}

/** The cosine and sine of `degrees`, exact at each quarter turn. */
function turn(degrees: number): [number, number] {
  const quarter = degrees / 90;
  if (Number.isInteger(quarter)) {
    const quarters: [number, number][] = [[1, 0], [0, 1], [-1, 0], [0, -1]];
    return quarters[((quarter % 4) + 4) % 4];
  }
  const radians = degrees * Math.PI / 180;
  return [Math.cos(radians), Math.sin(radians)];
}
