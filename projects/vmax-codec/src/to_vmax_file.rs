use crate::{
    to_contents_vmaxb_file_bytes, to_palette_png_file_bytes,
    to_palette_settings_vmaxpsb_file_bytes, to_scene_json_file_bytes,
};
use std::io::Result;
use vmax::VMaxSerdeFile;

/// Writes a [`VMaxSerdeFile`] back to a `.vmax` package. `write` receives each file's
/// name and bytes; like [`from_vmax_file`](crate::from_vmax_file) this keeps the codec free
/// of any filesystem dependency — the caller performs the actual writes.
pub fn to_vmax_file<W>(file: &VMaxSerdeFile, mut write: W) -> Result<()>
where
    W: FnMut(&str, &[u8]) -> Result<()>,
{
    write(
        "scene.json",
        &to_scene_json_file_bytes(&file.scene_json_file)?,
    )?;
    for (name, contents) in &file.contents_files {
        write(name, &to_contents_vmaxb_file_bytes(contents)?)?;
    }
    for (name, settings) in &file.palette_settings_files {
        write(name, &to_palette_settings_vmaxpsb_file_bytes(settings)?)?;
    }
    for (name, png) in &file.palette_png_files {
        write(name, &to_palette_png_file_bytes(png)?)?;
    }
    Ok(())
}
