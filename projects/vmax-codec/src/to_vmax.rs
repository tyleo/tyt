use crate::{to_palette_png_bytes, to_scene_json_bytes, to_vmaxb_bytes, to_vmaxpsb_bytes};
use std::io::Result;
use vmax::VMaxFile;

/// Writes a [`VMaxFile`] back to a `.vmax` package. `write` receives each file's
/// name and bytes; like [`from_vmax`](crate::from_vmax) this keeps the codec free
/// of any filesystem dependency — the caller performs the actual writes.
pub fn to_vmax<W>(file: &VMaxFile, mut write: W) -> Result<()>
where
    W: FnMut(&str, &[u8]) -> Result<()>,
{
    write("scene.json", &to_scene_json_bytes(&file.scene_json_file)?)?;
    for (name, contents) in &file.contents_vmaxb_files {
        write(name, &to_vmaxb_bytes(contents)?)?;
    }
    for (name, settings) in &file.palette_settings_vmaxpsb_files {
        write(name, &to_vmaxpsb_bytes(settings)?)?;
    }
    for (name, png) in &file.palette_png_files {
        write(name, &to_palette_png_bytes(png)?)?;
    }
    Ok(())
}
