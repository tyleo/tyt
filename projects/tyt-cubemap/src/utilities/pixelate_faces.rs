use crate::{Dependencies, Result};

const FACES: &[&str] = &["left", "right", "up", "down", "front", "back"];

/// Point-resizes six cube face images to the given height.
pub fn pixelate_faces(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    size: u32,
) -> Result<()> {
    for face in FACES {
        let in_path = format!("{base}-{face}.png");
        let out_path = format!("{out_base}-{face}.png");
        deps.exec_magick([
            in_path.as_str(),
            "-filter",
            "point",
            "-resize",
            &format!("x{size}"),
            &out_path,
        ])?;
    }
    Ok(())
}
