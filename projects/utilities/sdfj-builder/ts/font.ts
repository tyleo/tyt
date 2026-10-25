import { type Entry, entry } from "./entry.ts";
import { strictRecord } from "./strict_record.ts";

/** A font a `text` reads. */
export class Font {
  readonly #entry: Entry;

  /** The font whose `fonts` entry holds `fields`. */
  constructor(fields: Readonly<Record<string, unknown>>) {
    this.#entry = entry(fields);
    Object.freeze(this);
  }

  /** The `fonts` entry `font` writes. */
  static entry(font: Font): Entry {
    return font.#entry;
  }
}

/** The fonts the build sets, by name. */
export type Fonts = Readonly<Record<string, Font>>;

/**
 * The `fonts` global over the build's `fonts` entries. A missing name errors.
 */
export function fontsGlobal(fonts: readonly Entry[]): Fonts {
  const values = fonts.map((fields) => new Font(fields));
  const quoted = fonts.map(({ name }) => JSON.stringify(name));
  const set = quoted.length === 0 ? "none" : `only ${quoted.join(", ")}`;
  return strictRecord(
    values.map((font, index) => [fonts[index].name as string, () => font]),
    (name) =>
      `fonts has no font ${JSON.stringify(name)} because the build sets ${set}`,
  );
}
