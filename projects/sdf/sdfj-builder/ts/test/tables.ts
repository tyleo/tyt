import type { Entry } from "../entry.ts";
import type { Library } from "../library.ts";
import { material } from "../material.ts";
import type { Part } from "../part.ts";
import { sphere } from "../primitives.ts";
import { circle } from "../primitives2d.ts";
import { extrude } from "../profiles.ts";
import {
  type NamedValue,
  type SdfjDocument,
  sdfjDocument,
} from "../sdfj_document.ts";
import { sdfjJson } from "../sdfj_json.ts";
import type { Shape2d } from "../shape2d.ts";
import type { Shape3d } from "../shape3d.ts";
import { add, type Step } from "../step.ts";

/** A material for steps whose material a test ignores. */
export const white = material({ baseColor: "#FFFFFF" });

/** A material for tests that need a second one. */
export const black = material({ baseColor: "#000000" });

/** A shape for steps whose shape a test ignores. */
export const ball = sphere([0, 0, 0], 1);

/** The `shapes3d` entry of `ball`. */
export const BALL_ENTRY: Entry = {
  kind: "sphere",
  center: [0, 0, 0],
  radius: 1,
};

/** A 2D shape for profiles whose shape a test ignores. */
export const disk = circle([0, 0], 1);

/** The `shapes2d` entry of `disk`. */
export const DISK_ENTRY: Entry = { kind: "circle", center: [0, 0], radius: 1 };

/** The document of a model whose default export is `list`. */
export function document(list: readonly (Step | Part)[]): SdfjDocument {
  return sdfjDocument("model", list);
}

/** The library `name` whose document names each of `exports`. */
export function library(
  name: string,
  exports: readonly (readonly [string, NamedValue])[],
): Library {
  return { name, document: sdfjJson(sdfjDocument(name, [], exports)) };
}

/** The `shapes3d` table of a model that adds `shape`. */
export function shapes3d(shape: Shape3d): Entry[] {
  return document([add("shape", shape, white)]).shapes3d;
}

/** The `shapes2d` table of a model that adds `profile` extruded. */
export function shapes2d(profile: Shape2d): Entry[] {
  const shape = extrude(profile, { from: 0, to: 1 });
  return document([add("shape", shape, white)]).shapes2d;
}
