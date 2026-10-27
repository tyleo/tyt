import assert from "node:assert/strict";
import { Font, fontsGlobal } from "./font.ts";

Deno.test("fonts reads each font the build sets", () => {
  const sign = { kind: "installed", name: "sign", font: "Futura" };
  const label = { kind: "file", name: "label", path: "../fonts/plex.otf" };
  const fonts = fontsGlobal([label, sign]);
  assert.deepEqual(Font.entry(fonts.sign), sign);
  assert.deepEqual(Font.entry(fonts.label), label);
  assert.equal(fonts.sign, fonts.sign);
});

Deno.test("reading a font the build lacks errors", () => {
  assert.throws(() => fontsGlobal([]).sign, {
    message: 'fonts has no font "sign" because the build sets none',
  });
  assert.throws(
    () =>
      fontsGlobal([{ kind: "installed", name: "label", font: "Menlo" }]).sign,
    {
      message: 'fonts has no font "sign" because the build sets only "label"',
    },
  );
});
