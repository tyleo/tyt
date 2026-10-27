import * as check from "./check.ts";
import { Material } from "./material.ts";
import { Part, stepsAndParts } from "./part.ts";
import { Pattern } from "./pattern.ts";
import type { NamedValue } from "./sdfj_document.ts";
import { Shape2d } from "./shape2d.ts";
import { Shape3d } from "./shape3d.ts";
import { Step } from "./step.ts";

/** The steps and parts the model module `model` exports by default. */
export function modelList(
  model: Readonly<Record<string, unknown>>,
): (Step | Part)[] {
  if (!("default" in model)) {
    throw new Error("the model has no default export");
  }
  return stepsAndParts("the default export", model.default);
}

/** The named exports of the model module `model` other than its functions. */
export function namedExports(
  model: Readonly<Record<string, unknown>>,
): [string, NamedValue][] {
  return Object.entries(model)
    .filter(([name, value]) =>
      name !== "default" && typeof value !== "function"
    )
    .map(([name, value]) => [name, namedValue(`the export ${name}`, value)]);
}

/** `value` as a value a document can give a name. */
function namedValue(path: string, value: unknown): NamedValue {
  if (
    value instanceof Material || value instanceof Part ||
    value instanceof Pattern || value instanceof Shape2d ||
    value instanceof Shape3d || value instanceof Step
  ) {
    return value;
  }
  return check.fail(
    path,
    "a Material, a Part, a Pattern, a Shape2d, a Shape3d, a Step, or a function",
    value,
  );
}
