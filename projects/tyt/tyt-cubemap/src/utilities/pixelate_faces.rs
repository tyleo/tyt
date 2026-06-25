use crate::{Dependencies, Result, utilities};

/// Point-resizes six cube face images to the given height, optionally upscaling to
/// `output_size` afterwards to preserve hard edges at a larger resolution.
pub fn pixelate_faces(
    deps: &impl Dependencies,
    base: &str,
    out_base: &str,
    size: u32,
    output_size: Option<u32>,
) -> Result<()> {
    for face in utilities::C6X1_FACES {
        let in_path = format!("{base}-{face}.png");
        let out_path = format!("{out_base}-{face}.png");
        let size_arg = format!("x{size}");
        let output_size_arg;
        let mut args: Vec<&str> = vec![
            in_path.as_str(),
            "-filter",
            "point",
            "-resize",
            size_arg.as_str(),
        ];
        if let Some(output_size) = output_size {
            output_size_arg = format!("x{output_size}");
            args.extend(["-resize", output_size_arg.as_str()]);
        }
        args.push(out_path.as_str());
        deps.exec_magick(args)?;
    }
    Ok(())
}
