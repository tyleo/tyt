// Records a model as an `.sdfj` document. The builder runs as
// `main.ts <model> <output>`, with `--intern false` to keep an entry per value,
// and reads the libraries from the `libraries.json` beside this file.
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import url from "node:url";
import { jsonText } from "./json_text.ts";
import { modelList, namedExports } from "./model_list.ts";
import { putApiInScope } from "./put_api_in_scope.ts";
import { sdfjDocument } from "./sdfj_document.ts";
import { sdfjJson } from "./sdfj_json.ts";

const [modelPath, outputPath, ...extra] = process.argv.slice(2);
const intern = extra.length === 0;
if (
  modelPath === undefined || outputPath === undefined ||
  !(intern || (extra.length === 2 && extra.join(" ") === "--intern false"))
) {
  throw new Error(
    "the builder takes a model path, an output path, and an optional " +
      "`--intern false`",
  );
}
const librariesUrl = new URL("libraries.json", import.meta.url);
putApiInScope(JSON.parse(fs.readFileSync(librariesUrl, "utf8")));
const model = await import(url.pathToFileURL(path.resolve(modelPath)).href);
const document = sdfjDocument(
  path.parse(modelPath).name,
  modelList(model),
  namedExports(model),
  intern,
);
fs.writeFileSync(outputPath, `${jsonText(sdfjJson(document))}\n`);
