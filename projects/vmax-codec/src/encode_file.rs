use crate::{encode_contents, encode_palette_settings};
use vmax::{VMaxCodecFile, VMaxSerdeFile};

/// Encodes a [`VMaxCodecFile`] (decoded payloads) into a [`VMaxSerdeFile`] (raw
/// `.vmaxb` / `.vmaxpsb` parses), the inverse of
/// [`decode_file`](crate::decode_file), ready to write with
/// [`to_vmax`](crate::to_vmax). The scene graph and `palette*.png` color tables
/// carry over unchanged; each decoded object and palette is re-encoded.
pub fn encode_file(file: &VMaxCodecFile) -> VMaxSerdeFile {
    VMaxSerdeFile {
        scene_json_file: file.scene_json_file.clone(),
        contents_files: file
            .contents_files
            .iter()
            .map(|(name, contents)| (name.clone(), encode_contents(contents)))
            .collect(),
        palette_settings_files: file
            .palette_settings_files
            .iter()
            .map(|(name, palette)| (name.clone(), encode_palette_settings(palette)))
            .collect(),
        palette_png_files: file.palette_png_files.clone(),
    }
}
