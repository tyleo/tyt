use crate::{
    DecodeFont, Error, FindInstalledFont, ReadFontFile, Result, SdfjFontData, SdfjFontFile,
};
use sdfj::{SdfjFile, SdfjFont, SdfjShape2d};
use std::{collections::BTreeSet, path::Path};

/// The glyphs of each of `file`'s fonts, in order, for the characters of the
/// `text` shapes reading the font. A `file` font's path resolves against
/// `directory`, the document's directory.
pub fn load_sdfj_fonts<D: DecodeFont + FindInstalledFont + ReadFontFile>(
    dependencies: &D,
    file: &SdfjFile,
    directory: &Path,
) -> Result<Vec<SdfjFontData>> {
    let mut characters = vec![BTreeSet::new(); file.fonts.len()];
    for (index, shape) in file.shapes2d.iter().enumerate() {
        if let SdfjShape2d::Text { font, string, .. } = shape {
            characters
                .get_mut(*font)
                .ok_or_else(|| {
                    Error::Font(format!(
                        "`shapes2d[{index}]` reads the font {font}, past the end of `fonts`"
                    ))
                })?
                .extend(string.chars());
        }
    }

    file.fonts
        .iter()
        .zip(characters)
        .map(|(font, characters)| {
            let (name, font_file) = match font {
                SdfjFont::File { name, path } => {
                    let path = directory.join(path);
                    let data = dependencies.read_font_file(&path).map_err(|error| {
                        Error::Font(format!(
                            "the font `{name}` failed to read `{}`: {error}",
                            path.display()
                        ))
                    })?;

                    (name, SdfjFontFile { data, index: 0 })
                }

                SdfjFont::Installed { name, font } => {
                    let font_file = dependencies
                        .find_installed_font(font)
                        .map_err(|error| {
                            Error::Font(format!(
                                "the font `{name}` failed to open the installed `{font}`: {error}"
                            ))
                        })?
                        .ok_or_else(|| {
                            Error::Font(format!(
                                "the font `{name}` asks for `{font}`, which no installed face \
                                 takes as its family or full name"
                            ))
                        })?;

                    (name, font_file)
                }
            };

            dependencies
                .decode_font(&font_file, &characters)
                .map_err(|error| Error::Font(format!("the font `{name}` does not decode: {error}")))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        DecodeFont, FindInstalledFont, ReadFontFile, SdfjFontData, SdfjFontFile, SdfjGlyph,
        load_sdfj_fonts,
    };
    use sdfj::{SDFJ_VERSION, SdfjFile, SdfjFont, SdfjNames, SdfjShape2d};
    use std::{
        collections::{BTreeMap, BTreeSet},
        io::{Error as IOError, ErrorKind, Result as IOResult},
        path::Path,
    };

    /// The font file `/repo/models/../fonts/plex.otf` and the installed family
    /// `Test Sans`, decoding to a glyph for each character asked for with the
    /// face's index as the units per em.
    struct Fonts;

    impl ReadFontFile for Fonts {
        fn read_font_file(&self, path: &Path) -> IOResult<Vec<u8>> {
            match path.to_str() {
                Some("/repo/models/../fonts/plex.otf") => Ok(b"plex".to_vec()),
                _ => Err(IOError::from(ErrorKind::NotFound)),
            }
        }
    }

    impl FindInstalledFont for Fonts {
        fn find_installed_font(&self, font: &str) -> Result<Option<SdfjFontFile>, String> {
            Ok((font == "Test Sans").then(|| SdfjFontFile {
                data: b"test sans".to_vec(),
                index: 2,
            }))
        }
    }

    impl DecodeFont for Fonts {
        fn decode_font(
            &self,
            file: &SdfjFontFile,
            characters: &BTreeSet<char>,
        ) -> Result<SdfjFontData, String> {
            Ok(SdfjFontData {
                units_per_em: f64::from(file.index),
                cap_height: None,
                glyphs: characters
                    .iter()
                    .map(|character| (*character, SdfjGlyph::default()))
                    .collect(),
                kerning: BTreeMap::new(),
            })
        }
    }

    fn text(font: usize, string: &str) -> SdfjShape2d {
        SdfjShape2d::Text {
            font,
            string: string.to_owned(),
            size: 0.1,
            align: None,
        }
    }

    /// The fonts of a document in `/repo/models` holding `fonts` and
    /// `shapes2d`, each as its units per em and characters, or the error
    /// message.
    fn load(
        fonts: Vec<SdfjFont>,
        shapes2d: Vec<SdfjShape2d>,
    ) -> Result<Vec<(f64, String)>, String> {
        let file = SdfjFile {
            version: SDFJ_VERSION,
            shapes3d: Vec::new(),
            shapes2d,
            fonts,
            materials: Vec::new(),
            shades: Vec::new(),
            patterns: Vec::new(),
            steps: Vec::new(),
            objects: Vec::new(),
            nodes: Vec::new(),
            root_nodes: Vec::new(),
            names: SdfjNames::default(),
        };

        load_sdfj_fonts(&Fonts, &file, Path::new("/repo/models"))
            .map(|fonts| {
                fonts
                    .into_iter()
                    .map(|font| (font.units_per_em, font.glyphs.into_keys().collect()))
                    .collect()
            })
            .map_err(|error| error.to_string())
    }

    fn file(path: &str) -> SdfjFont {
        SdfjFont::File {
            name: "label".to_owned(),
            path: path.to_owned(),
        }
    }

    fn installed(font: &str) -> SdfjFont {
        SdfjFont::Installed {
            name: "sign".to_owned(),
            font: font.to_owned(),
        }
    }

    #[test]
    fn each_font_decodes_the_characters_of_the_texts_reading_it() {
        assert_eq!(
            load(
                vec![file("../fonts/plex.otf"), installed("Test Sans")],
                vec![text(1, "OPEN"), text(0, "Mon"), text(1, "NOW")],
            )
            .unwrap(),
            [(0.0, "Mno".to_owned()), (2.0, "ENOPW".to_owned())]
        );
    }

    #[test]
    fn a_font_that_fails_to_read_or_match_errors() {
        let error = load(vec![file("plex.otf")], Vec::new()).unwrap_err();
        assert!(
            error.starts_with("the font `label` failed to read `/repo/models/plex.otf`"),
            "{error}"
        );

        assert_eq!(
            load(vec![installed("Test Serif")], Vec::new()).unwrap_err(),
            "the font `sign` asks for `Test Serif`, which no installed face takes as its family \
             or full name"
        );
    }
}
