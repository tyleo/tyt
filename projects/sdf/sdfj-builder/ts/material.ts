import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";

/** Surface properties for the cells a step fills or recolors. */
export class Material {
  readonly #entry: Entry;

  /** The material whose `materials` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `materials` entry `material` writes. */
  static entry(material: Material): Entry {
    return material.#entry;
  }
}

/** A custom property holding a whole number or a vector of them. */
export class IntValue {
  readonly #entry: Entry;

  /** The property value whose tagged form holds `value`. */
  constructor(value: number | readonly number[]) {
    this.#entry = { kind: "int", value };
    Object.freeze(this);
  }

  /** The tagged form `int` writes. */
  static entry(int: IntValue): Entry {
    return int.#entry;
  }
}

/** A custom property holding any JSON value. */
export class JsonValue {
  readonly #entry: Entry;

  /** The property value whose tagged form holds `value`. */
  constructor(value: unknown) {
    this.#entry = { kind: "json", value };
    Object.freeze(this);
  }

  /** The tagged form `json` writes. */
  static entry(json: JsonValue): Entry {
    return json.#entry;
  }
}

/** The properties of a material by name. */
export interface Properties {
  /** The surface color as sRGB `#RRGGBB`, or `#RRGGBBAA` with alpha. */
  baseColor: string;

  /** The color the surface emits as sRGB `#RRGGBB`. */
  emissiveColor?: string;

  /** The multiplier on `emissiveColor`. */
  emissiveStrength?: number;

  /** The index of refraction. */
  ior?: number;

  /** How metallic the surface is, from 0 to 1. */
  metallic?: number;

  /** How strongly ambient occlusion darkens the surface. */
  occlusionStrength?: number;

  /** How rough the surface is, from 0 to 1. */
  roughness?: number;

  /** How much light passes through the surface, from 0 to 1. */
  transmission?: number;

  /** A custom property. */
  [name: string]: Value | undefined;
}

/** The value of a custom property. */
export type Value =
  | boolean
  | number
  | string
  | number[]
  | IntValue
  | JsonValue;

/** The named properties holding colors. */
const COLOR_PROPERTIES = ["baseColor", "emissiveColor"];

/** The named properties holding numbers. */
const NUMBER_PROPERTIES = [
  "emissiveStrength",
  "ior",
  "metallic",
  "occlusionStrength",
  "roughness",
  "transmission",
];

/** The material holding `properties`. */
export function material(properties: Properties): Material {
  if (!check.isPlainObject(properties)) {
    check.fail("material properties", "an object", properties);
  }
  check.text("material baseColor", properties.baseColor);
  const written = Object.entries(properties)
    .filter(([, value]) => value !== undefined)
    .map(([name, value]) => [name, property(`material ${name}`, name, value)]);
  return new Material({
    kind: "material",
    properties: Object.fromEntries(written),
  });
}

/** The custom property value holding the whole number or numbers `value`. */
export function int(value: number | number[]): IntValue {
  return new IntValue(
    Array.isArray(value)
      ? check.list("int value", value, "an array of numbers", check.finite)
      : check.finite("int value", value),
  );
}

/** The custom property value holding the JSON value `value`. */
export function json(value: unknown): JsonValue {
  return new JsonValue(jsonCopy("json value", value, new Set()));
}

/** The form the property `name` writes for `value`. */
function property(path: string, name: string, value: unknown): unknown {
  if (COLOR_PROPERTIES.includes(name)) {
    return check.text(path, value);
  }
  if (NUMBER_PROPERTIES.includes(name)) {
    return check.finite(path, value);
  }
  if (typeof value === "boolean" || typeof value === "string") {
    return value;
  }
  if (typeof value === "number") {
    return check.finite(path, value);
  }
  if (Array.isArray(value)) {
    return check.list(path, value, "an array of numbers", check.finite);
  }
  if (value instanceof IntValue) {
    return IntValue.entry(value);
  }
  if (value instanceof JsonValue) {
    return JsonValue.entry(value);
  }
  return check.fail(
    path,
    "a boolean, a number, a string, an array of numbers, an IntValue, or a JsonValue",
    value,
  );
}

/** A copy of `value` holding only JSON, inside the objects in `ancestors`. */
function jsonCopy(
  path: string,
  value: unknown,
  ancestors: Set<object>,
): unknown {
  if (
    value === null || typeof value === "boolean" || typeof value === "string"
  ) {
    return value;
  }
  if (typeof value === "number") {
    return check.finite(path, value);
  }
  if (!Array.isArray(value) && !check.isPlainObject(value)) {
    return check.fail(path, "a JSON value", value);
  }
  if (ancestors.has(value)) {
    throw new Error(`${path} holds itself`);
  }
  ancestors.add(value);
  const copy = Array.isArray(value)
    ? Array.from(
      value,
      (item: unknown, index) => jsonCopy(`${path}[${index}]`, item, ancestors),
    )
    : Object.fromEntries(
      Object.entries(value).map((
        [key, item],
      ) => [key, jsonCopy(`${path}.${key}`, item, ancestors)]),
    );
  ancestors.delete(value);
  return copy;
}
