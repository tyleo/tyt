import * as api from "./api.ts";
import { type Library, libraryGlobals } from "./library.ts";
import { varsGlobal } from "./vars.ts";

/**
 * Puts the API on `globalThis`, with `lib` and `mat` reading `libraries` and
 * `vars` reading the build variables `vars`.
 */
export function putApiInScope(
  libraries: readonly Library[],
  vars: Readonly<Record<string, unknown>>,
): void {
  const globals: Pick<
    typeof globalThis,
    keyof typeof api | "lib" | "mat" | "vars"
  > = {
    ...api,
    ...libraryGlobals(libraries),
    vars: varsGlobal(vars),
  };
  Object.assign(globalThis, globals);
}
