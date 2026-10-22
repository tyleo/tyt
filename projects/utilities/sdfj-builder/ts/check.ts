import type { Axes, Axis, AxisRotation, Side, Vec2, Vec3 } from "./types.ts";

/** A check of the value at a path, returning the value as its type. */
export type Check<T> = (path: string, value: unknown) => T;

/** Throws the error for a `value` at `path` that is not `expectation`. */
export function fail(path: string, expectation: string, value: unknown): never {
  throw new Error(`${path} must be ${expectation}, not ${describe(value)}`);
}

/** Whether `value` is an object literal or a parsed JSON object. */
export function isPlainObject(
  value: unknown,
): value is Readonly<Record<string, unknown>> {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

/** `value` as a finite number. */
export function finite(path: string, value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    fail(path, "a finite number", value);
  }
  return value;
}

/** `value` as a whole number above zero. */
export function count(path: string, value: unknown): number {
  if (!Number.isInteger(value) || (value as number) < 1) {
    fail(path, "a whole number above zero", value);
  }
  return value as number;
}

/** `value` as a string. */
export function text(path: string, value: unknown): string {
  if (typeof value !== "string") {
    fail(path, "a string", value);
  }
  return value;
}

/** `value` as a boolean. */
export function boolean(path: string, value: unknown): boolean {
  if (typeof value !== "boolean") {
    fail(path, "true or false", value);
  }
  return value;
}

/** A copy of `value` as a `Vec2`. */
export function vec2(path: string, value: unknown): Vec2 {
  const [u, v] = numbers(path, value, 2, "a Vec2 of 2 finite numbers");
  return [u, v];
}

/** A copy of `value` as a `Vec3`. */
export function vec3(path: string, value: unknown): Vec3 {
  const [x, y, z] = numbers(path, value, 3, "a Vec3 of 3 finite numbers");
  return [x, y, z];
}

/** A copy of `value` as an `AxisRotation`. */
export function axisRotation(path: string, value: unknown): AxisRotation {
  if (!Array.isArray(value) || value.length !== 2) {
    fail(path, "an AxisRotation of an axis and finite degrees", value);
  }
  return [axis(`${path}[0]`, value[0]), finite(`${path}[1]`, value[1])];
}

/** `value` as a `Vec2`, with one number standing for both components. */
export function numberOrVec2(path: string, value: unknown): Vec2 {
  return typeof value === "number"
    ? [finite(path, value), value]
    : numbers(path, value, 2, "a finite number or a Vec2") as Vec2;
}

/** `value` as a `Vec3`, with one number standing for all three components. */
export function numberOrVec3(path: string, value: unknown): Vec3 {
  return typeof value === "number"
    ? [finite(path, value), value, value]
    : numbers(path, value, 3, "a finite number or a Vec3") as Vec3;
}

/** A copy of the array `value` with `item` checking each element. */
export function list<T>(
  path: string,
  value: unknown,
  expectation: string,
  item: Check<T>,
): T[] {
  if (!Array.isArray(value)) {
    fail(path, expectation, value);
  }
  return Array.from(
    value,
    (element: unknown, index) => item(`${path}[${index}]`, element),
  );
}

/** `value` checked by `check`, or `undefined` when the model left it out. */
export function optional<T>(
  path: string,
  value: unknown,
  check: Check<T>,
): T | undefined {
  return value === undefined ? undefined : check(path, value);
}

/** `value` as an instance of `type`. */
export function instance<T>(
  path: string,
  value: unknown,
  type: abstract new (...args: never[]) => T,
): T {
  if (!(value instanceof type)) {
    fail(path, article(type.name), value);
  }
  return value;
}

/** The options object `value` passed to `call`, holding only `keys`. */
export function knownOptions(
  call: string,
  value: unknown,
  keys: readonly string[],
): Readonly<Record<string, unknown>> {
  if (value === undefined) {
    return {};
  }
  if (!isPlainObject(value)) {
    fail(`${call} options`, "an object", value);
  }
  const unknownKey = Object.keys(value).find((key) => !keys.includes(key));
  if (unknownKey !== undefined) {
    throw new Error(`${call} takes no option ${JSON.stringify(unknownKey)}`);
  }
  return value;
}

/** `value` as an `Axis`. */
export function axis(path: string, value: unknown): Axis {
  return oneOf(path, value, ["x", "y", "z"]);
}

/** `value` as the `Axes` of a 3D mirror. */
export function axes(path: string, value: unknown): Axes {
  return oneOf(path, value, ["x", "y", "z", "xy", "xz", "yz", "xyz"]);
}

/** `value` as the axes of a 2D mirror. */
export function axes2d(path: string, value: unknown): "u" | "v" | "uv" {
  return oneOf(path, value, ["u", "v", "uv"]);
}

/** `value` as the caps of an arc. */
export function caps(path: string, value: unknown): "flat" | "round" {
  return oneOf(path, value, ["flat", "round"]);
}

/** `value` as a `Side`. */
export function side(path: string, value: unknown): Side {
  return oneOf(path, value, ["+x", "-x", "+y", "-y", "+z", "-z"]);
}

/** `value` as one of `values`. */
function oneOf<T extends string>(
  path: string,
  value: unknown,
  values: readonly T[],
): T {
  if (!values.includes(value as T)) {
    const quoted = values.map((option) => JSON.stringify(option));
    fail(path, `one of ${quoted.join(", ")}`, value);
  }
  return value as T;
}

/** A copy of the array `value` holding `length` finite numbers. */
function numbers(
  path: string,
  value: unknown,
  length: number,
  expectation: string,
): number[] {
  if (!Array.isArray(value) || value.length !== length) {
    fail(path, expectation, value);
  }
  return Array.from(
    value,
    (item: unknown) =>
      Number.isFinite(item) ? item as number : fail(path, expectation, value),
  );
}

/** `name` after the article its first letter takes. */
function article(name: string): string {
  return /^[aeiou]/i.test(name) ? `an ${name}` : `a ${name}`;
}

/** `value` as an error message shows it. */
function describe(value: unknown): string {
  if (typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    return Object.is(value, -0) ? "-0" : String(value);
  }
  if (typeof value === "bigint") {
    return `${value}n`;
  }
  if (typeof value === "function") {
    return "a function";
  }
  if (Array.isArray(value)) {
    const items = Array.from(value.slice(0, 8), describe);
    return `[${[...items, ...(value.length > 8 ? ["..."] : [])].join(", ")}]`;
  }
  if (isPlainObject(value)) {
    return "an object";
  }
  if (typeof value === "object" && value !== null) {
    return article(Object.getPrototypeOf(value)?.constructor?.name ?? "object");
  }
  return String(value);
}
