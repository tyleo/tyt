use crate::{
    CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson, Result,
    to_contents_vmaxb_file_bytes, to_history_vmaxhb_file_bytes, to_history_vmaxhvsb_file_bytes,
    to_history_vmaxhvsc_file_bytes, to_image_png_file_bytes, to_palette_png_file_bytes,
    to_palette_settings_vmaxpsb_file_bytes, to_scene_json_file_bytes,
    to_selection_vmaxb_file_bytes,
};
use vmax::VMaxFile;

/// Writes a [`VMaxFile`] back to a `.vmax` package, encoding each file
/// through `dependencies`, the inverse of
/// [`from_vmax_package`](crate::from_vmax_package).
///
/// # Arguments
/// * `write` - receives each file's package-relative name and bytes and
///   performs the actual write, creating any subdirectory a `QuickLook/` name
///   implies.
pub fn to_vmax_package<D, W>(dependencies: &D, file: &VMaxFile, mut write: W) -> Result<()>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
    W: FnMut(&str, &[u8]) -> Result<()>,
{
    write(
        "scene.json",
        &to_scene_json_file_bytes(dependencies, &file.scene_json_file),
    )?;
    for (name, contents) in &file.contents_files {
        write(name, &to_contents_vmaxb_file_bytes(dependencies, contents)?)?;
    }
    for (name, settings) in &file.palette_settings_files {
        write(
            name,
            &to_palette_settings_vmaxpsb_file_bytes(dependencies, settings)?,
        )?;
    }
    for (name, png) in &file.palette_png_files {
        write(name, &to_palette_png_file_bytes(dependencies, png)?)?;
    }
    for (name, history) in &file.history_vmaxhb_files {
        write(name, &to_history_vmaxhb_file_bytes(dependencies, history)?)?;
    }
    for (name, history) in &file.history_vmaxhvsb_files {
        write(
            name,
            &to_history_vmaxhvsb_file_bytes(dependencies, history)?,
        )?;
    }
    for (name, history) in &file.history_vmaxhvsc_files {
        write(
            name,
            &to_history_vmaxhvsc_file_bytes(dependencies, history)?,
        )?;
    }
    for (name, selection) in &file.selection_vmaxb_files {
        write(name, &to_selection_vmaxb_file_bytes(selection)?)?;
    }
    // QuickLook previews, reconstructing each role's `QuickLook/` path.
    if let Some(thumbnail) = &file.thumbnail_png {
        write(
            "QuickLook/Thumbnail.png",
            &to_image_png_file_bytes(dependencies, thumbnail)?,
        )?;
    }
    for (data, image) in &file.contents_vmax_pngs {
        write(
            &format!("QuickLook/{data}.png"),
            &to_image_png_file_bytes(dependencies, image)?,
        )?;
    }
    for (id, image) in &file.group_pngs {
        write(
            &format!("QuickLook/{id}.png"),
            &to_image_png_file_bytes(dependencies, image)?,
        )?;
    }
    Ok(())
}
