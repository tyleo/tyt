use crate::{
    DecodeFont, DecodeSdfjJson, Deflate, EncodeSdfjJson, FindInstalledFont, Inflate, ReadFontFile,
    SdfjFontData, SdfjFontFile, SdfjGlyph, SdfjGlyphContour, SdfjGlyphSegment,
};
use flate2::{Compression, read::DeflateDecoder, write::DeflateEncoder};
use fontdb::{Database, Family, ID, Query};
use sdfj::SdfjFile;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Result as IOResult, Write},
    path::Path,
    sync::OnceLock,
};
use ttf_parser::{
    Face, GlyphId, OutlineBuilder, PlatformId, Tag,
    gpos::{PairAdjustment, PositioningSubtable},
    name::Name,
    name_id,
};

/// The dependencies the `impl` feature provides.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl DecodeFont for DependenciesImpl {
    fn decode_font(
        &self,
        file: &SdfjFontFile,
        characters: &BTreeSet<char>,
    ) -> Result<SdfjFontData, String> {
        let face = Face::parse(&file.data, file.index).map_err(|error| error.to_string())?;
        let cap_height = face
            .capital_height()
            .filter(|height| *height > 0)
            .or_else(|| {
                let top = face.glyph_bounding_box(face.glyph_index('H')?)?.y_max;
                (top > 0).then_some(top)
            })
            .map(f64::from);

        let mut glyph_ids = BTreeMap::new();
        let mut glyphs = BTreeMap::new();
        for &character in characters {
            // Glyph 0 stands in for a character the face lacks.
            let Some(glyph_id) = face
                .glyph_index(character)
                .filter(|glyph_id| glyph_id.0 != 0)
            else {
                continue;
            };
            let advance = face
                .glyph_hor_advance(glyph_id)
                .ok_or_else(|| format!("`{character}` has no advance"))?;

            let mut builder = ContourBuilder::default();
            face.outline_glyph(glyph_id, &mut builder);
            glyph_ids.insert(character, glyph_id);
            glyphs.insert(
                character,
                SdfjGlyph {
                    advance: f64::from(advance),
                    contours: builder.contours,
                },
            );
        }

        Ok(SdfjFontData {
            units_per_em: f64::from(face.units_per_em()),
            cap_height,
            glyphs,
            kerning: kerning(&face, &glyph_ids),
        })
    }
}

impl DecodeSdfjJson for DependenciesImpl {
    fn decode_sdfj_json(&self, bytes: &[u8]) -> Result<SdfjFile, String> {
        serde_json::from_slice(bytes).map_err(|error| error.to_string())
    }
}

impl EncodeSdfjJson for DependenciesImpl {
    fn encode_sdfj_json(&self, file: &SdfjFile) -> Result<Vec<u8>, String> {
        serde_json::to_vec(file).map_err(|error| error.to_string())
    }

    fn encode_sdfj_json_pretty(&self, file: &SdfjFile) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(file).map_err(|error| error.to_string())
    }
}

impl Deflate for DependenciesImpl {
    fn deflate(&self, bytes: &[u8]) -> Vec<u8> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(bytes)
            .expect("write to Vec is infallible");
        encoder.finish().expect("flush to Vec is infallible")
    }
}

impl Inflate for DependenciesImpl {
    fn inflate(&self, stream: &[u8]) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        DeflateDecoder::new(stream)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }
}

impl FindInstalledFont for DependenciesImpl {
    fn find_installed_font(&self, font: &str) -> Result<Option<SdfjFontFile>, String> {
        // The scan of the installed fonts runs once, when a document first asks
        // for one.
        static INSTALLED: OnceLock<Database> = OnceLock::new();
        let database = INSTALLED.get_or_init(|| {
            let mut database = Database::new();
            database.load_system_fonts();
            database
        });

        find_font(database, font)
    }
}

impl ReadFontFile for DependenciesImpl {
    fn read_font_file(&self, path: &Path) -> IOResult<Vec<u8>> {
        fs::read(path)
    }
}

/// Collects a glyph's contours in font units.
#[derive(Default)]
struct ContourBuilder {
    contours: Vec<SdfjGlyphContour>,
}

impl ContourBuilder {
    fn push(&mut self, segment: SdfjGlyphSegment) {
        self.contours
            .last_mut()
            .expect("a contour starts with a move")
            .segments
            .push(segment);
    }
}

impl OutlineBuilder for ContourBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.contours.push(SdfjGlyphContour {
            start: point(x, y),
            segments: Vec::new(),
        });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.push(SdfjGlyphSegment::Line { to: point(x, y) });
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.push(SdfjGlyphSegment::Quadratic {
            control: point(x1, y1),
            to: point(x, y),
        });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.push(SdfjGlyphSegment::Cubic {
            control1: point(x1, y1),
            control2: point(x2, y2),
            to: point(x, y),
        });
    }

    fn close(&mut self) {}
}

/// The nonzero kerning `face` makes between each pair of `glyph_ids`'
/// characters. The face's GPOS `kern` lookups set it, and a face without them
/// takes its `kern` table.
fn kerning(face: &Face, glyph_ids: &BTreeMap<char, GlyphId>) -> BTreeMap<(char, char), f64> {
    let kern = Tag::from_bytes(b"kern");
    let lookups: Vec<_> = face
        .tables()
        .gpos
        .map(|gpos| {
            let lookup_indices: BTreeSet<_> = gpos
                .features
                .into_iter()
                .filter(|feature| feature.tag == kern)
                .flat_map(|feature| feature.lookup_indices)
                .collect();
            lookup_indices
                .into_iter()
                .filter_map(|lookup_index| gpos.lookups.get(lookup_index))
                .collect()
        })
        .unwrap_or_default();

    let pair_kerning = |left: GlyphId, right: GlyphId| -> i32 {
        if lookups.is_empty() {
            return face.tables().kern.map_or(0, |table| {
                table
                    .subtables
                    .into_iter()
                    .filter(|subtable| subtable.horizontal && !subtable.variable)
                    .filter_map(|subtable| subtable.glyphs_kerning(left, right))
                    .map(i32::from)
                    .sum()
            });
        }

        // Each lookup adds the first of its subtables that holds the pair.
        lookups
            .iter()
            .filter_map(|lookup| {
                lookup
                    .subtables
                    .into_iter::<PositioningSubtable>()
                    .find_map(|subtable| pair_advance(subtable, left, right))
            })
            .map(i32::from)
            .sum()
    };

    glyph_ids
        .iter()
        .flat_map(|(&left, &left_id)| {
            glyph_ids
                .iter()
                .map(move |(&right, &right_id)| ((left, right), pair_kerning(left_id, right_id)))
        })
        .filter(|(_, kerning)| *kerning != 0)
        .map(|(pair, kerning)| (pair, f64::from(kerning)))
        .collect()
}

/// The adjustment `subtable` adds to `left`'s advance when `right` follows it,
/// or `None` when the subtable does not hold the pair.
fn pair_advance(subtable: PositioningSubtable, left: GlyphId, right: GlyphId) -> Option<i16> {
    let PositioningSubtable::Pair(pair) = subtable else {
        return None;
    };
    let coverage_index = pair.coverage().get(left)?;
    let (first, _) = match pair {
        PairAdjustment::Format1 { sets, .. } => sets.get(coverage_index)?.get(right)?,

        PairAdjustment::Format2 {
            classes, matrix, ..
        } => matrix.get((classes.0.get(left), classes.1.get(right)))?,
    };

    Some(first.x_advance)
}

/// The point at `x` and `y` in font units.
fn point(x: f32, y: f32) -> [f64; 2] {
    [f64::from(x), f64::from(y)]
}

/// The face of `database` that [`FindInstalledFont`] picks for `font`.
fn find_font(database: &Database, font: &str) -> Result<Option<SdfjFontFile>, String> {
    let Some(face_id) = installed_face_id(database, font) else {
        return Ok(None);
    };

    database
        .with_face_data(face_id, |data, index| SdfjFontFile {
            data: data.to_vec(),
            index,
        })
        .map(Some)
        .ok_or_else(|| "the face's file failed to open".to_owned())
}

/// The installed face whose family or full name matches `font`, ignoring case.
/// A family picks its regular face.
fn installed_face_id(database: &Database, font: &str) -> Option<ID> {
    let font = font.to_lowercase();

    let family = database
        .faces()
        .flat_map(|face| &face.families)
        .map(|(family, _)| family)
        .find(|family| family.to_lowercase() == font);

    if let Some(family) = family {
        let query = Query {
            families: &[Family::Name(family)],
            ..Query::default()
        };
        return Some(database.query(&query).expect("a family holds a face"));
    }

    database
        .faces()
        .find(|face| {
            database.with_face_data(face.id, |data, index| {
                Face::parse(data, index).is_ok_and(|face| {
                    face.names().into_iter().any(|name| {
                        name.name_id == name_id::FULL_NAME
                            && name_text(&name)
                                .is_some_and(|full_name| full_name.to_lowercase() == font)
                    })
                })
            }) == Some(true)
        })
        .map(|face| face.id)
}

/// The text of `name`, which a font holds in UTF-16 or Mac Roman.
fn name_text(name: &Name) -> Option<String> {
    if name.platform_id == PlatformId::Macintosh && name.encoding_id == 0 {
        Some(
            name.name
                .iter()
                .map(|&byte| match byte.checked_sub(0x80) {
                    Some(high) => MAC_ROMAN[usize::from(high)],
                    None => char::from(byte),
                })
                .collect(),
        )
    } else {
        name.to_string()
    }
}

/// The characters of the Mac Roman bytes from 0x80 up.
const MAC_ROMAN: [char; 128] = [
    '\u{00C4}', '\u{00C5}', '\u{00C7}', '\u{00C9}', '\u{00D1}', '\u{00D6}', '\u{00DC}', '\u{00E1}',
    '\u{00E0}', '\u{00E2}', '\u{00E4}', '\u{00E3}', '\u{00E5}', '\u{00E7}', '\u{00E9}', '\u{00E8}',
    '\u{00EA}', '\u{00EB}', '\u{00ED}', '\u{00EC}', '\u{00EE}', '\u{00EF}', '\u{00F1}', '\u{00F3}',
    '\u{00F2}', '\u{00F4}', '\u{00F6}', '\u{00F5}', '\u{00FA}', '\u{00F9}', '\u{00FB}', '\u{00FC}',
    '\u{2020}', '\u{00B0}', '\u{00A2}', '\u{00A3}', '\u{00A7}', '\u{2022}', '\u{00B6}', '\u{00DF}',
    '\u{00AE}', '\u{00A9}', '\u{2122}', '\u{00B4}', '\u{00A8}', '\u{2260}', '\u{00C6}', '\u{00D8}',
    '\u{221E}', '\u{00B1}', '\u{2264}', '\u{2265}', '\u{00A5}', '\u{00B5}', '\u{2202}', '\u{2211}',
    '\u{220F}', '\u{03C0}', '\u{222B}', '\u{00AA}', '\u{00BA}', '\u{03A9}', '\u{00E6}', '\u{00F8}',
    '\u{00BF}', '\u{00A1}', '\u{00AC}', '\u{221A}', '\u{0192}', '\u{2248}', '\u{2206}', '\u{00AB}',
    '\u{00BB}', '\u{2026}', '\u{00A0}', '\u{00C0}', '\u{00C3}', '\u{00D5}', '\u{0152}', '\u{0153}',
    '\u{2013}', '\u{2014}', '\u{201C}', '\u{201D}', '\u{2018}', '\u{2019}', '\u{00F7}', '\u{25CA}',
    '\u{00FF}', '\u{0178}', '\u{2044}', '\u{20AC}', '\u{2039}', '\u{203A}', '\u{FB01}', '\u{FB02}',
    '\u{2021}', '\u{00B7}', '\u{201A}', '\u{201E}', '\u{2030}', '\u{00C2}', '\u{00CA}', '\u{00C1}',
    '\u{00CB}', '\u{00C8}', '\u{00CD}', '\u{00CE}', '\u{00CF}', '\u{00CC}', '\u{00D3}', '\u{00D4}',
    '\u{F8FF}', '\u{00D2}', '\u{00DA}', '\u{00DB}', '\u{00D9}', '\u{0131}', '\u{02C6}', '\u{02DC}',
    '\u{00AF}', '\u{02D8}', '\u{02D9}', '\u{02DA}', '\u{00B8}', '\u{02DD}', '\u{02DB}', '\u{02C7}',
];

#[cfg(test)]
mod tests {
    use super::find_font;
    use crate::{
        DecodeFont, Deflate, DependenciesImpl, Inflate, SdfjFontFile, SdfjGlyph, SdfjGlyphContour,
        SdfjGlyphSegment, TestGlyph, TestKerning, test_font,
    };
    use fontdb::Database;

    #[test]
    fn deflate_round_trips_and_ignores_trailing_bytes() {
        let text = b"{\"version\":1}";
        let mut stream = DependenciesImpl.deflate(text);
        assert!(stream.len() < text.len() + 8);
        assert_eq!(DependenciesImpl.inflate(&stream).unwrap(), text);

        stream.extend_from_slice(b"PK\x01\x02trailing central directory");
        assert_eq!(DependenciesImpl.inflate(&stream).unwrap(), text);
    }

    #[test]
    fn inflate_rejects_a_malformed_stream() {
        assert!(DependenciesImpl.inflate(&[0xFF, 0xFF, 0xFF]).is_err());
    }

    const H: TestGlyph = (
        'H',
        600,
        &[&[
            (0, 0, true),
            (0, 700, true),
            (500, 700, true),
            (500, 0, true),
        ]],
    );

    /// A font file named `full_name` at `weight` in the family `family`.
    fn font(family: &str, full_name: &str, weight: u16) -> Vec<u8> {
        test_font(family, full_name, weight, 700, &[H], &[], &[])
    }

    #[test]
    fn an_installed_font_matches_a_family_or_a_full_name_ignoring_case() {
        let mut database = Database::new();
        database.load_font_data(font("Test Sans", "Test Sans Bold", 700));
        database.load_font_data(font("Test Sans", "Test Sans Regular", 400));

        let data = |name: &str| find_font(&database, name).unwrap().map(|font| font.data);
        assert_eq!(
            data("test sans"),
            Some(font("Test Sans", "Test Sans Regular", 400))
        );
        assert_eq!(
            data("TEST SANS BOLD"),
            Some(font("Test Sans", "Test Sans Bold", 700))
        );
        assert_eq!(data("Test Serif"), None);
    }

    #[test]
    fn a_face_decodes_the_characters_it_holds_and_measures_its_cap_height() {
        let decode = |cap_height: i16, characters: &str| {
            let file = SdfjFontFile {
                data: test_font(
                    "Test Sans",
                    "Test Sans Regular",
                    400,
                    cap_height,
                    &[H],
                    &[],
                    &[],
                ),
                index: 0,
            };
            DependenciesImpl
                .decode_font(&file, &characters.chars().collect())
                .unwrap()
        };

        let font = decode(650, "HÉ");
        assert_eq!(font.cap_height, Some(650.0));
        assert_eq!(font.glyphs.keys().collect::<Vec<_>>(), [&'H']);
        assert_eq!(
            font.glyphs[&'H'],
            SdfjGlyph {
                advance: 600.0,
                contours: vec![SdfjGlyphContour {
                    start: [0.0, 0.0],
                    segments: [[0.0, 700.0], [500.0, 700.0], [500.0, 0.0], [0.0, 0.0]]
                        .map(|to| SdfjGlyphSegment::Line { to })
                        .to_vec(),
                }],
            }
        );

        // Without a cap height, the top of `H` stands in, decoded or not.
        assert_eq!(decode(0, "").cap_height, Some(700.0));
    }

    #[test]
    fn a_face_kerns_by_its_gpos_lookups_or_else_its_kern_table() {
        let kerning = |kern: &[TestKerning], gpos: &[TestKerning]| {
            let glyphs = [('A', 600, &[][..]), H, ('V', 600, &[][..])];
            let file = SdfjFontFile {
                data: test_font(
                    "Test Sans",
                    "Test Sans Regular",
                    400,
                    700,
                    &glyphs,
                    kern,
                    gpos,
                ),
                index: 0,
            };
            let kerning = DependenciesImpl
                .decode_font(&file, &"AHV".chars().collect())
                .unwrap()
                .kerning;
            kerning.into_iter().collect::<Vec<_>>()
        };

        let kern = [('A', 'V', -80), ('V', 'A', -60)];
        assert_eq!(
            kerning(&kern, &[]),
            [(('A', 'V'), -80.0), (('V', 'A'), -60.0)]
        );

        // A pair the font kerns by 0 takes no entry.
        let gpos = [('A', 'H', 0), ('A', 'V', -120), ('H', 'V', 10)];
        assert_eq!(
            kerning(&kern, &gpos),
            [(('A', 'V'), -120.0), (('H', 'V'), 10.0)]
        );
    }
}
