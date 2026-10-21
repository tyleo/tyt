import * as check from "./check.ts";
import { type Entry, entry } from "./entry.ts";
import { jsonText } from "./json_text.ts";
import { libraryNames } from "./library.ts";
import { Material } from "./material.ts";
import { Part } from "./part.ts";
import { Pattern } from "./pattern.ts";
import { Shades } from "./shades.ts";
import { Shape2d } from "./shape2d.ts";
import { Shape3d } from "./shape3d.ts";
import { Step } from "./step.ts";
import type { AxisRotation, Vec3 } from "./types.ts";

/** The sdfj format version the builder writes. */
const SDFJ_VERSION = 1;

/** An `.sdfj` document as JSON values. */
export interface SdfjDocument {
  /** The format version. */
  version: number;

  /** The `Shape3d` entries. */
  shapes3d: Entry[];

  /** The `Shape2d` entries. */
  shapes2d: Entry[];

  /** The `Material` entries. */
  materials: Entry[];

  /** The `shades` calls. */
  shades: Entry[];

  /** The `Pattern` entries. */
  patterns: Entry[];

  /** The `Step` entries. */
  steps: Entry[];

  /** An object per part whose list holds steps. */
  objects: Entry[];

  /** A node per part. */
  nodes: Entry[];

  /** The nodes at the top of the hierarchy. */
  rootNodes: number[];

  /** The names of the entries, by names table. */
  names: SdfjNames;
}

/** The names a document gives its entries. */
export interface SdfjNames {
  /** Names of `Shape3d` entries. */
  shapes3d: Record<string, number>;

  /** Names of `Shape2d` entries. */
  shapes2d: Record<string, number>;

  /** Names of `Material` entries. */
  materials: Record<string, number>;

  /** Names of `Pattern` entries. */
  patterns: Record<string, number>;

  /** Names of `Step` entries. */
  steps: Record<string, number>;

  /** Names of the nodes parts write. */
  parts: Record<string, number>;
}

/** A value a document can give a name. */
export type NamedValue = Material | Part | Pattern | Shape2d | Shape3d | Step;

/**
 * The document of the model `name` whose default export is `list` and whose
 * named exports are `exports`. A value met again reuses its entry, and with
 * `intern` so does a value whose entry writes the same text. Each entry takes
 * the names of the exports holding it and the names it holds in its library.
 */
export function sdfjDocument(
  name: string,
  list: readonly (Step | Part)[],
  exports: readonly (readonly [string, NamedValue])[] = [],
  intern = true,
): SdfjDocument {
  const writer = new SdfjWriter(intern);
  writer.document.rootNodes.push(
    writer.node(name, undefined, undefined, undefined, list),
  );
  for (const [, value] of exports) {
    writer.named(value);
  }
  writer.document.names = writer.names(exports);
  return writer.document;
}

/** A value with an entry in one of the document's tables. */
type TableValue = Material | Pattern | Shades | Shape2d | Shape3d | Step;

type NamesTable = keyof SdfjNames;

/** The document so far and the index each written value took. */
class SdfjWriter {
  readonly document: SdfjDocument = {
    version: SDFJ_VERSION,
    shapes3d: [],
    shapes2d: [],
    materials: [],
    shades: [],
    patterns: [],
    steps: [],
    objects: [],
    nodes: [],
    rootNodes: [],
    names: {
      shapes3d: {},
      shapes2d: {},
      materials: {},
      patterns: {},
      steps: {},
      parts: {},
    },
  };

  readonly #indices = new Map<TableValue | Part, number>();

  /** Each table's entry indices by entry text, when the writer interns. */
  readonly #textIndices: Map<Entry[], Map<string, number>> | undefined;

  constructor(intern: boolean) {
    this.#textIndices = intern ? new Map() : undefined;
  }

  /** Writes the entry `value` holds. */
  named(value: NamedValue): void {
    if (value instanceof Part) {
      this.#part(value);
    } else {
      this.#index(value);
    }
  }

  /** The names of the written entries. */
  names(exports: readonly (readonly [string, NamedValue])[]): SdfjNames {
    const tables = new Map<NamesTable, Map<string, number>>();
    const add = (name: string, value: TableValue | Part) => {
      const table = namesTable(value);
      if (table === undefined) {
        return;
      }
      const index = this.#indices.get(value) as number;
      let names = tables.get(table);
      if (names === undefined) {
        names = new Map();
        tables.set(table, names);
      }
      const taken = names.get(name);
      if (taken !== undefined && taken !== index) {
        const values = `${value.constructor.name}s`;
        throw new Error(
          `two different ${values} take the name ${JSON.stringify(name)}`,
        );
      }
      names.set(name, index);
    };
    for (const value of this.#indices.keys()) {
      for (const name of libraryNames(value)) {
        add(name, value);
      }
    }
    for (const [name, value] of exports) {
      add(name, value);
    }
    const sorted = (table: NamesTable) =>
      Object.fromEntries(
        [...tables.get(table) ?? []].sort(([a], [b]) => a < b ? -1 : 1),
      );
    return {
      shapes3d: sorted("shapes3d"),
      shapes2d: sorted("shapes2d"),
      materials: sorted("materials"),
      patterns: sorted("patterns"),
      steps: sorted("steps"),
      parts: sorted("parts"),
    };
  }

  /** Writes a node, and an object when `list` holds steps. */
  node(
    name: string,
    pivot: Vec3 | undefined,
    offset: Vec3 | undefined,
    rotation: readonly AxisRotation[] | undefined,
    list: readonly (Step | Part)[],
  ): number {
    const steps: number[] = [];
    const childNodes: number[] = [];
    const listed = new Set<Step>();
    for (const item of list) {
      if (item instanceof Step) {
        checkOnto(item, listed);
        listed.add(item);
        steps.push(this.#index(item));
      } else {
        childNodes.push(this.#part(item));
      }
    }
    const childObjects = steps.length === 0
      ? []
      : [this.document.objects.push({ name, steps }) - 1];
    const node = entry({
      name,
      pivot,
      offset,
      rotation: rotation?.length
        ? rotation.map(([axis, degrees]) => ({ axis, degrees }))
        : undefined,
      childObjects,
      childNodes,
    });
    return this.document.nodes.push(node) - 1;
  }

  /** The index of the node `part` writes. */
  #part(part: Part): number {
    let index = this.#indices.get(part);
    if (index === undefined) {
      const { name, pivot, offset, rotation, list } = Part.entry(part);
      index = this.node(name, pivot, offset, rotation, list);
      this.#indices.set(part, index);
    }
    return index;
  }

  /** The index of the entry `value` writes, after the entries it references. */
  #index(value: TableValue): number {
    let index = this.#indices.get(value);
    if (index === undefined) {
      const [table, fields] = this.#tableEntry(value);
      index = this.#push(table, this.#write(fields) as Entry);
      this.#indices.set(value, index);
    }
    return index;
  }

  /** The index `entry` takes in `table`, reusing an entry of the same text. */
  #push(table: Entry[], entry: Entry): number {
    if (this.#textIndices === undefined) {
      return table.push(entry) - 1;
    }
    let indices = this.#textIndices.get(table);
    if (indices === undefined) {
      indices = new Map();
      this.#textIndices.set(table, indices);
    }
    const text = jsonText(entry);
    let index = indices.get(text);
    if (index === undefined) {
      index = table.push(entry) - 1;
      indices.set(text, index);
    }
    return index;
  }

  /** `value` with an index in place of each value it holds. */
  #write(value: unknown): unknown {
    if (
      value instanceof Material || value instanceof Pattern ||
      value instanceof Shades || value instanceof Shape2d ||
      value instanceof Shape3d || value instanceof Step
    ) {
      return this.#index(value);
    }
    if (Array.isArray(value)) {
      return value.map((item) => this.#write(item));
    }
    if (check.isPlainObject(value)) {
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, this.#write(item)]),
      );
    }
    return value;
  }

  /** The table `value` writes into and the entry it writes. */
  #tableEntry(value: TableValue): [Entry[], Entry] {
    if (value instanceof Material) {
      return [this.document.materials, Material.entry(value)];
    }
    if (value instanceof Pattern) {
      return [this.document.patterns, Pattern.entry(value)];
    }
    if (value instanceof Shades) {
      return [this.document.shades, Shades.entry(value)];
    }
    if (value instanceof Shape2d) {
      return [this.document.shapes2d, Shape2d.entry(value)];
    }
    if (value instanceof Shape3d) {
      return [this.document.shapes3d, Shape3d.entry(value)];
    }
    return [this.document.steps, Step.entry(value)];
  }
}

/** Throws unless each step in the `onto` of `step` is in `listed`. */
function checkOnto(step: Step, listed: ReadonlySet<Step>): void {
  const { kind, name, onto } = Step.entry(step);
  for (const ontoStep of (onto ?? []) as Step[]) {
    if (!listed.has(ontoStep)) {
      throw new Error(
        `${kind} ${JSON.stringify(name)} onto must list steps earlier in ` +
          `its list, not ${JSON.stringify(Step.entry(ontoStep).name)}`,
      );
    }
  }
}

/** The names table holding the names of `value`'s entry, if any. */
function namesTable(value: TableValue | Part): NamesTable | undefined {
  if (value instanceof Material) {
    return "materials";
  }
  if (value instanceof Part) {
    return "parts";
  }
  if (value instanceof Pattern) {
    return "patterns";
  }
  if (value instanceof Shape2d) {
    return "shapes2d";
  }
  if (value instanceof Shape3d) {
    return "shapes3d";
  }
  if (value instanceof Step) {
    return "steps";
  }
  return undefined;
}
