import { strictRecord } from "./strict_record.ts";

/** The build variables by name, each holding any JSON value. */
// deno-lint-ignore no-explicit-any
export type Vars = Readonly<Record<string, any>>;

/** The `vars` global over the build variables `vars`. A missing name errors. */
export function varsGlobal(vars: Readonly<Record<string, unknown>>): Vars {
  const quoted = Object.keys(vars).map((name) => JSON.stringify(name));
  const set = quoted.length === 0 ? "none" : `only ${quoted.join(", ")}`;
  return strictRecord(
    Object.entries(vars).map(([name, value]) => [name, () => value]),
    (name) =>
      `vars has no variable ${
        JSON.stringify(name)
      } because the build sets ${set}`,
  );
}
