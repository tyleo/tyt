use crate::{Dependencies, Result};

/// Pads an image to a square canvas with transparent background.
pub fn square(deps: &impl Dependencies, in_path: &str, out_path: &str) -> Result<()> {
    deps.exec_magick([
        in_path,
        "-background",
        "none",
        "-gravity",
        "center",
        "-extent",
        "%[fx:max(w,h)]x%[fx:max(w,h)]",
        out_path,
    ])?;
    Ok(())
}
