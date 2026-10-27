import * as check from "./check.ts";

/**
 * The compact JSON text of `value`. A number takes its shortest round-trip
 * form, and `-0` keeps its sign.
 */
export function jsonText(value: unknown): string {
  if (typeof value === "number") {
    return Object.is(value, -0)
      ? "-0"
      : JSON.stringify(check.finite("a document number", value));
  }
  if (
    value === null || typeof value === "boolean" || typeof value === "string"
  ) {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map((item) => jsonText(item)).join(",")}]`;
  }
  if (check.isPlainObject(value)) {
    const members = Object.entries(value).map(([key, item]) =>
      `${JSON.stringify(key)}:${jsonText(item)}`
    );
    return `{${members.join(",")}}`;
  }
  return check.fail("a document value", "JSON", value);
}
