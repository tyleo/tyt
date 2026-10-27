/** A table entry holding values where the written document holds indices. */
export type Entry = Readonly<Record<string, unknown>>;

/** `fields` without the keys that hold `undefined`. */
export function entry(fields: Readonly<Record<string, unknown>>): Entry {
  return Object.fromEntries(
    Object.entries(fields).filter(([, value]) => value !== undefined),
  );
}
