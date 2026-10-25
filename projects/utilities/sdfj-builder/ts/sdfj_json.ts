import type { SdfjDocument, SdfjNames } from "./sdfj_document.ts";

/** An `.sdfj` document as JSON, which can leave out any table or names map. */
export type SdfjJson =
  & Pick<SdfjDocument, "version">
  & Partial<Omit<SdfjDocument, "version" | "names">>
  & { names?: Partial<SdfjNames> };

/** `document` without its empty tables and names maps. */
export function sdfjJson(document: SdfjDocument): SdfjJson {
  const names = withoutEmpty(document.names);
  return withoutEmpty({ ...document, names }) as SdfjJson;
}

/** `json` with an empty table or names map in place of each one it lacks. */
export function sdfjDocumentFromJson(json: SdfjJson): SdfjDocument {
  const names = json.names ?? {};
  return {
    version: json.version,
    shapes3d: json.shapes3d ?? [],
    shapes2d: json.shapes2d ?? [],
    fonts: json.fonts ?? [],
    materials: json.materials ?? [],
    shades: json.shades ?? [],
    patterns: json.patterns ?? [],
    steps: json.steps ?? [],
    objects: json.objects ?? [],
    nodes: json.nodes ?? [],
    rootNodes: json.rootNodes ?? [],
    names: {
      shapes3d: names.shapes3d ?? {},
      shapes2d: names.shapes2d ?? {},
      materials: names.materials ?? {},
      patterns: names.patterns ?? {},
      steps: names.steps ?? {},
      parts: names.parts ?? {},
    },
  };
}

/** `record` without its empty arrays and objects. */
function withoutEmpty(record: object): Record<string, unknown> {
  return Object.fromEntries(
    Object.entries(record).filter(([, value]) =>
      typeof value !== "object" || Object.keys(value).length > 0
    ),
  );
}
