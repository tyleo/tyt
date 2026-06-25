use crate::{
    Result, to_contents_vmaxb_file_bytes, to_history_vmaxhb_file_bytes,
    to_history_vmaxhvsb_file_bytes, to_history_vmaxhvsc_file_bytes, to_palette_png_file_bytes,
    to_palette_settings_vmaxpsb_file_bytes, to_quick_look_png_file_bytes, to_scene_json_file_bytes,
    to_selection_vmaxb_file_bytes,
};
use vmax::VMaxSerdeFile;

/// Writes a [`VMaxSerdeFile`] back to a `.vmax` package, the inverse of
/// [`from_vmax_file`](crate::from_vmax_file). `write` receives each file's
/// package-relative name and bytes and performs the actual writes (creating
/// any subdirectory a `QuickLook/` name implies).
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
    for (name, history) in &file.history_vmaxhb_files {
        write(name, &to_history_vmaxhb_file_bytes(history)?)?;
    }
    for (name, history) in &file.history_vmaxhvsb_files {
        write(name, &to_history_vmaxhvsb_file_bytes(history)?)?;
    }
    for (name, history) in &file.history_vmaxhvsc_files {
        write(name, &to_history_vmaxhvsc_file_bytes(history)?)?;
    }
    for (name, selection) in &file.selection_vmaxb_files {
        write(name, &to_selection_vmaxb_file_bytes(selection)?)?;
    }
    for (name, png) in &file.quick_look_png_files {
        write(name, &to_quick_look_png_file_bytes(png)?)?;
    }
    Ok(())
}
