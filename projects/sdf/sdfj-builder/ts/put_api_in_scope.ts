import * as api from "./api.ts";
import type { Entry } from "./entry.ts";
import { fontsGlobal } from "./font.ts";
import { type Library, libraryGlobals } from "./library.ts";
import { varsGlobal } from "./vars.ts";

/**
 * Puts the API on `globalThis`, with `lib` and `mat` reading `libraries`,
 * `fonts` reading the `fonts` entries `fonts`, and `vars` reading the build
 * variables `vars`.
 */
export function putApiInScope(
  libraries: readonly Library[],
  fonts: readonly Entry[],
  vars: Readonly<Record<string, unknown>>,
): void {
  const globals: Pick<
    typeof globalThis,
    keyof typeof api | "fonts" | "lib" | "mat" | "vars"
  > = {
    ...api,
    ...libraryGlobals(libraries),
    fonts: fontsGlobal(fonts),
    vars: varsGlobal(vars),
  };
  Object.assign(globalThis, globals);
}
