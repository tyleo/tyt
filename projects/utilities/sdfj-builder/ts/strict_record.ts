/**
 * A frozen record of `getters`, where a later getter wins a name. Reading a
 * name the record lacks throws the error `missing` describes.
 */
export function strictRecord<T>(
  getters: readonly (readonly [string, () => T])[],
  missing: (name: string) => string,
): Readonly<Record<string, T>> {
  const record: Record<string, T> = Object.create(null);
  for (const [name, get] of getters) {
    Object.defineProperty(record, name, {
      configurable: true,
      enumerable: true,
      get,
    });
  }
  return new Proxy(Object.freeze(record), {
    get(target, key) {
      if (typeof key === "string" && !Object.hasOwn(target, key)) {
        throw new Error(missing(key));
      }
      return Reflect.get(target, key);
    },
  });
}
