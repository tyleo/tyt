import type { Entry } from "./entry.ts";
import { Material } from "./material.ts";
import { Part } from "./part.ts";
import { Pattern } from "./pattern.ts";
import type { SdfjDocument, SdfjNames } from "./sdfj_document.ts";
import { sdfjDocumentFromJson, type SdfjJson } from "./sdfj_json.ts";
import { Shades } from "./shades.ts";
import { Shape2d } from "./shape2d.ts";
import { Shape3d } from "./shape3d.ts";
import { Step } from "./step.ts";
import { strictRecord } from "./strict_record.ts";
import type { Axis, AxisRotation, Vec3 } from "./types.ts";

/** A library vxl hands the builder. */
export interface Library {
  /** The name a build lists the library by. */
  readonly name: string;

  /** The library's document, which vxl checked against the sdfj format. */
  readonly document: SdfjJson;
}

/** The entries the libraries name, by names table and name. */
export interface Lib {
  /** The named `Shape3d`s. */
  readonly shapes3d: Readonly<Record<string, Shape3d>>;

  /** The named `Shape2d`s. */
  readonly shapes2d: Readonly<Record<string, Shape2d>>;

  /** The named `Material`s. */
  readonly materials: Readonly<Record<string, Material>>;

  /** The named `Pattern`s. */
  readonly patterns: Readonly<Record<string, Pattern>>;

  /** The named `Step`s. */
  readonly steps: Readonly<Record<string, Step>>;

  /** The named `Part`s. */
  readonly parts: Readonly<Record<string, Part>>;
}

/** The names a value read from a library holds in the library's document. */
const LIBRARY_NAMES = new WeakMap<object, readonly string[]>();

/** The names `value` holds in the library it came from. */
export function libraryNames(value: object): readonly string[] {
  return LIBRARY_NAMES.get(value) ?? [];
}

/**
 * The `lib` and `mat` globals over `libraries`. A later library wins a name.
 * Reading a name no library holds errors.
 */
export function libraryGlobals(
  libraries: readonly Library[],
): { lib: Lib; mat: Lib["materials"] } {
  const readers = libraries.map((library) => new LibraryReader(library));
  const quoted = libraries.map(({ name }) => JSON.stringify(name));
  const searched = quoted.length === 0
    ? "because the build reads no library"
    : `in the ${quoted.length === 1 ? "library" : "libraries"} ${
      quoted.join(", ")
    }`;
  const table = <T>(path: string, names: keyof SdfjNames, noun: string) =>
    strictRecord<T>(
      readers.flatMap((reader) => reader.named(names)) as [string, () => T][],
      (name) => `${path} has no ${noun} ${JSON.stringify(name)} ${searched}`,
    );
  const lib: Lib = {
    shapes3d: table("lib.shapes3d", "shapes3d", "3D shape"),
    shapes2d: table("lib.shapes2d", "shapes2d", "2D shape"),
    materials: table("lib.materials", "materials", "material"),
    patterns: table("lib.patterns", "patterns", "pattern"),
    steps: table("lib.steps", "steps", "step"),
    parts: table("lib.parts", "parts", "part"),
  };
  return {
    lib: strictRecord(
      Object.entries(lib).map(([name, value]) => [name, () => value]),
      (name) => `lib has no table ${JSON.stringify(name)}`,
    ) as Lib,
    mat: table("mat", "materials", "material"),
  };
}

/** A table of a document whose entries read as values. */
type ValueTable =
  | "shapes3d"
  | "shapes2d"
  | "materials"
  | "shades"
  | "patterns"
  | "steps"
  | "nodes";

/** Each table's keys that hold indices, with the table each key points into. */
const REFERENCES: Readonly<
  Record<
    Exclude<ValueTable, "nodes">,
    Readonly<Record<string, ValueTable>>
  >
> = {
  shapes3d: {
    base: "shapes3d",
    cutters: "shapes3d",
    profile: "shapes2d",
    shape: "shapes3d",
    shapes: "shapes3d",
  },
  shapes2d: {
    base: "shapes2d",
    cutters: "shapes2d",
    shape: "shapes2d",
    shapes: "shapes2d",
  },
  materials: { shades: "shades" },
  shades: { base: "materials" },
  patterns: {
    accents: "materials",
    base: "materials",
    border: "materials",
    materials: "materials",
  },
  steps: {
    material: "materials",
    pattern: "patterns",
    shape: "shapes3d",
    within: "shapes3d",
  },
};

/** The class each table's entries read as. */
const VALUES = {
  shapes3d: Shape3d,
  shapes2d: Shape2d,
  materials: Material,
  shades: Shades,
  patterns: Pattern,
  steps: Step,
};

/** A `nodes` entry of a document. */
interface NodeEntry {
  readonly name: string;
  readonly pivot?: Vec3;
  readonly offset?: Vec3;
  readonly rotation?: readonly {
    readonly axis: Axis;
    readonly degrees: number;
  }[];
  readonly childObjects: readonly number[];
  readonly childNodes: readonly number[];
}

/** An `objects` entry of a document. */
interface ObjectEntry {
  readonly name: string;
  readonly steps: readonly number[];
}

/** Reads one library's entries as values, each entry once. */
class LibraryReader {
  readonly #name: string;

  readonly #document: SdfjDocument;

  /** The value each read entry took, keyed by `table[index]`. */
  readonly #values = new Map<string, object>();

  /** The node each read object belongs to. */
  readonly #objectNodes = new Map<number, number>();

  constructor({ name, document }: Library) {
    this.#name = name;
    this.#document = sdfjDocumentFromJson(document);
  }

  /** Each name of the names table `names` with the getter of its value. */
  named(names: keyof SdfjNames): [string, () => object][] {
    const table = names === "parts" ? "nodes" : names;
    return Object.entries(this.#document.names[names]).map((
      [name, index],
    ) => [name, () => this.#value(table, index)]);
  }

  /** The value of the entry at `index` of `table`. */
  #value(table: ValueTable, index: number): object {
    const key = `${table}[${index}]`;
    let value = this.#values.get(key);
    if (value === undefined) {
      value = table === "nodes"
        ? this.#part(index)
        : new VALUES[table](this.#fields(table, index));
      this.#values.set(key, value);
      const names = this.#names(table, index);
      if (names.length > 0) {
        LIBRARY_NAMES.set(value, names);
      }
    }
    return value;
  }

  /** The entry at `index` of `table` with a value in place of each index. */
  #fields(
    table: Exclude<ValueTable, "nodes">,
    index: number,
  ): Readonly<Record<string, unknown>> {
    const references = REFERENCES[table];
    const fields = this.#entry(table, index) as Entry;
    return Object.fromEntries(
      Object.entries(fields).map(([key, value]) => {
        if (!Object.hasOwn(references, key)) {
          return [key, value];
        }
        const referenced = references[key];
        return [
          key,
          Array.isArray(value)
            ? value.map((item: number) => this.#value(referenced, item))
            : this.#value(referenced, value as number),
        ];
      }),
    );
  }

  /** The part the node at `index` writes. */
  #part(index: number): Part {
    const where = `${this.#where} nodes[${index}]`;
    const node = this.#entry("nodes", index) as NodeEntry;
    if (node.childObjects.length > 1) {
      throw new Error(
        `${where} holds ${node.childObjects.length} objects, and a part holds at most one`,
      );
    }
    const steps = node.childObjects.flatMap((objectIndex) => {
      const object = this.#entry("objects", objectIndex) as ObjectEntry;
      const owner = this.#objectNodes.get(objectIndex);
      if (owner !== undefined) {
        throw new Error(
          `${where} shares objects[${objectIndex}] with nodes[${owner}], and each part holds its own object`,
        );
      }
      if (object.name !== node.name) {
        throw new Error(
          `${where} holds objects[${objectIndex}] named ${
            JSON.stringify(object.name)
          }, and a part's object takes the part's name ${
            JSON.stringify(node.name)
          }`,
        );
      }
      if (object.steps.length === 0) {
        throw new Error(
          `${where} holds objects[${objectIndex}] with no steps, and a part writes an object only for its steps`,
        );
      }
      this.#objectNodes.set(objectIndex, index);
      return object.steps.map((step) => this.#value("steps", step) as Step);
    });
    const parts = node.childNodes.map((child) =>
      this.#value("nodes", child) as Part
    );
    return new Part({
      name: node.name,
      pivot: node.pivot,
      offset: node.offset,
      rotation: node.rotation?.map((
        { axis, degrees },
      ): AxisRotation => [axis, degrees]),
      list: [...steps, ...parts],
    });
  }

  /** The entry at `index` of `table`, which vxl's checks guarantee. */
  #entry(table: ValueTable | "objects", index: number): unknown {
    const entry = this.#document[table][index];
    if (entry === undefined) {
      throw new Error(`${this.#where} has no ${table}[${index}]`);
    }
    return entry;
  }

  /** The names the entry at `index` of `table` holds. */
  #names(table: ValueTable, index: number): string[] {
    if (table === "shades") {
      return [];
    }
    const names = this.#document.names[
      table === "nodes" ? "parts" : table
    ];
    return Object.keys(names).filter((name) => names[name] === index);
  }

  /** The start of an error about the library. */
  get #where(): string {
    return `library ${JSON.stringify(this.#name)}`;
  }
}
