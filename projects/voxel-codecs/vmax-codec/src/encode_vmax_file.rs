use crate::{encode_contents_vmaxb_file, encode_palette_settings_vmaxpsb_file};
use vmax::{VMaxCodecFile, VMaxSerdeFile};

/// Encodes a [`VMaxCodecFile`] into a [`VMaxSerdeFile`], the inverse of
/// [`decode_vmax_file`](crate::decode_vmax_file).
pub fn encode_vmax_file(file: &VMaxCodecFile) -> VMaxSerdeFile {
    VMaxSerdeFile {
        scene_json_file: file.scene_json_file.clone(),
        contents_files: file
            .contents_files
            .iter()
            .map(|(name, contents)| (name.clone(), encode_contents_vmaxb_file(contents)))
            .collect(),
        palette_settings_files: file
            .palette_settings_files
            .iter()
            .map(|(name, palette)| (name.clone(), encode_palette_settings_vmaxpsb_file(palette)))
            .collect(),
        palette_png_files: file.palette_png_files.clone(),
        history_vmaxhb_files: file.history_vmaxhb_files.clone(),
        history_vmaxhvsb_files: file.history_vmaxhvsb_files.clone(),
        history_vmaxhvsc_files: file.history_vmaxhvsc_files.clone(),
        selection_vmaxb_files: file.selection_vmaxb_files.clone(),
        quick_look_png_files: file.quick_look_png_files.clone(),
    }
}
