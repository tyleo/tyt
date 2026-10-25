use crate::{Result, from_sdfj_file};
use branded_id::IdVec;
use sdfcore::{SdfFontData, SdfGlyph, SdfGlyphContour, SdfGlyphSegment, SdfMain};
use sdfj_codec::{
    DecodeFont, DecodeSdfjJson, FindInstalledFont, Inflate, ReadFontFile, SdfjFontData, SdfjGlyph,
    SdfjGlyphContour, SdfjGlyphSegment, from_sdfj_or_sdfjz_file_bytes, load_sdfj_fonts,
};
use std::path::Path;
use ty_math::TyVector2F64;

/// Decodes a `.sdfj` or `.sdfjz` document into an [`SdfMain`] and loads its
/// fonts. A `file` font's path resolves against `directory`, the document's
/// directory.
pub fn from_sdfj_bytes_with_fonts<
    D: DecodeFont + DecodeSdfjJson + FindInstalledFont + Inflate + ReadFontFile,
>(
    dependencies: &D,
    bytes: &[u8],
    directory: &Path,
) -> Result<SdfMain> {
    let file = from_sdfj_or_sdfjz_file_bytes(dependencies, bytes)?;
    let main = from_sdfj_file(&file)?;
    let font_data = load_sdfj_fonts(dependencies, &file, directory)?
        .into_iter()
        .map(sdf_font_data_from_sdfj_font_data)
        .collect();

    Ok(main.with_font_data(IdVec::from_vec(font_data)))
}

fn sdf_font_data_from_sdfj_font_data(font_data: SdfjFontData) -> SdfFontData {
    SdfFontData {
        units_per_em: font_data.units_per_em,
        cap_height: font_data.cap_height,
        glyphs: font_data
            .glyphs
            .into_iter()
            .map(|(character, glyph)| (character, sdf_glyph_from_sdfj_glyph(glyph)))
            .collect(),
    }
}

fn sdf_glyph_from_sdfj_glyph(glyph: SdfjGlyph) -> SdfGlyph {
    SdfGlyph {
        advance: glyph.advance,
        contours: glyph
            .contours
            .into_iter()
            .map(sdf_glyph_contour_from_sdfj_glyph_contour)
            .collect(),
    }
}

fn sdf_glyph_contour_from_sdfj_glyph_contour(contour: SdfjGlyphContour) -> SdfGlyphContour {
    SdfGlyphContour {
        start: TyVector2F64::from_array(contour.start),
        segments: contour
            .segments
            .into_iter()
            .map(sdf_glyph_segment_from_sdfj_glyph_segment)
            .collect(),
    }
}

fn sdf_glyph_segment_from_sdfj_glyph_segment(segment: SdfjGlyphSegment) -> SdfGlyphSegment {
    match segment {
        SdfjGlyphSegment::Cubic {
            control1,
            control2,
            to,
        } => SdfGlyphSegment::Cubic {
            control1: TyVector2F64::from_array(control1),
            control2: TyVector2F64::from_array(control2),
            to: TyVector2F64::from_array(to),
        },

        SdfjGlyphSegment::Line { to } => SdfGlyphSegment::Line {
            to: TyVector2F64::from_array(to),
        },

        SdfjGlyphSegment::Quadratic { control, to } => SdfGlyphSegment::Quadratic {
            control: TyVector2F64::from_array(control),
            to: TyVector2F64::from_array(to),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{EVERY_KIND_SDFJ, codec::from_sdfj_bytes_with_fonts};
    use sdfcore::{SdfGlyph, SdfGlyphContour, SdfGlyphSegment};
    use sdfj::SdfjFile;
    use sdfj_codec::{
        DecodeFont, DecodeSdfjJson, DependenciesImpl, FindInstalledFont, Inflate, ReadFontFile,
        SdfjFontData, SdfjFontFile, SdfjGlyph, SdfjGlyphContour, SdfjGlyphSegment,
    };
    use std::{collections::BTreeSet, io::Result as IOResult, path::Path};
    use ty_math::TyVector2F64;

    /// The codec impl's JSON, with each font file holding its path or its
    /// installed name and decoding to that many units per em and one segment
    /// of each kind for each character.
    struct Fonts;

    impl DecodeSdfjJson for Fonts {
        fn decode_sdfj_json(&self, bytes: &[u8]) -> Result<SdfjFile, String> {
            DependenciesImpl.decode_sdfj_json(bytes)
        }
    }

    impl Inflate for Fonts {
        fn inflate(&self, stream: &[u8]) -> Result<Vec<u8>, String> {
            DependenciesImpl.inflate(stream)
        }
    }

    impl ReadFontFile for Fonts {
        fn read_font_file(&self, path: &Path) -> IOResult<Vec<u8>> {
            Ok(path.to_string_lossy().into_owned().into_bytes())
        }
    }

    impl FindInstalledFont for Fonts {
        fn find_installed_font(&self, font: &str) -> Result<Option<SdfjFontFile>, String> {
            Ok(Some(SdfjFontFile {
                data: font.as_bytes().to_vec(),
                index: 0,
            }))
        }
    }

    impl DecodeFont for Fonts {
        fn decode_font(
            &self,
            file: &SdfjFontFile,
            characters: &BTreeSet<char>,
        ) -> Result<SdfjFontData, String> {
            let glyph = SdfjGlyph {
                advance: 600.0,
                contours: vec![SdfjGlyphContour {
                    start: [0.0, 0.0],
                    segments: vec![
                        SdfjGlyphSegment::Line { to: [0.0, 1.0] },
                        SdfjGlyphSegment::Quadratic {
                            control: [1.0, 2.0],
                            to: [2.0, 1.0],
                        },
                        SdfjGlyphSegment::Cubic {
                            control1: [3.0, 1.0],
                            control2: [3.0, 0.0],
                            to: [2.0, 0.0],
                        },
                    ],
                }],
            };

            Ok(SdfjFontData {
                units_per_em: file.data.len() as f64,
                cap_height: Some(700.0),
                glyphs: characters
                    .iter()
                    .map(|character| (*character, glyph.clone()))
                    .collect(),
            })
        }
    }

    #[test]
    fn each_font_loads_the_glyphs_of_its_texts_in_the_order_of_the_table() {
        let main =
            from_sdfj_bytes_with_fonts(&Fonts, EVERY_KIND_SDFJ.as_bytes(), Path::new("models"))
                .unwrap();
        let font_data = main.font_data().unwrap();

        let fonts = font_data
            .iter()
            .map(|font| (font.units_per_em, font.glyphs.keys().collect::<String>()))
            .collect::<Vec<_>>();
        assert_eq!(
            fonts,
            [
                ("models/../fonts/plex.otf".len() as f64, "ENOP".to_owned()),
                ("Futura".len() as f64, " 4Aeils".to_owned()),
            ]
        );

        let point = |u: f64, v: f64| TyVector2F64::new(u, v);
        assert_eq!(
            font_data.iter().next().unwrap().glyphs[&'O'],
            SdfGlyph {
                advance: 600.0,
                contours: vec![SdfGlyphContour {
                    start: point(0.0, 0.0),
                    segments: vec![
                        SdfGlyphSegment::Line {
                            to: point(0.0, 1.0)
                        },
                        SdfGlyphSegment::Quadratic {
                            control: point(1.0, 2.0),
                            to: point(2.0, 1.0),
                        },
                        SdfGlyphSegment::Cubic {
                            control1: point(3.0, 1.0),
                            control2: point(3.0, 0.0),
                            to: point(2.0, 0.0),
                        },
                    ],
                }],
            }
        );
    }
}
