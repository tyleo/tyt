use crate::{Dependencies, Result};
use clap::Parser;

/// Pixelates (point-resizes) an image.
#[derive(Clone, Debug, Parser)]
pub struct Pixelate {
    /// Base name for the input image (`{base}.png`).
    #[arg(value_name = "base")]
    base: String,

    /// Output base name. Defaults to `{base}-px`.
    #[arg(value_name = "out-base")]
    out_base: Option<String>,

    /// Target height in pixels.
    #[arg(value_name = "size", short, long, default_value_t = 256)]
    size: u32,

    /// Final output height in pixels. When set, the image is point-resized up to this
    /// height after pixelation, preserving hard edges at a larger resolution.
    #[arg(value_name = "output-size", long)]
    output_size: Option<u32>,
}

impl Pixelate {
    pub fn execute(self, deps: impl Dependencies) -> Result<()> {
        let out_base = self.out_base.unwrap_or_else(|| format!("{}-px", self.base));
        let in_path = format!("{}.png", self.base);
        let out_path = format!("{out_base}.png");
        let size_arg = format!("x{}", self.size);
        let output_size_arg;
        let mut args: Vec<&str> = vec![
            in_path.as_str(),
            "-filter",
            "point",
            "-resize",
            size_arg.as_str(),
        ];
        if let Some(output_size) = self.output_size {
            output_size_arg = format!("x{output_size}");
            args.extend(["-resize", output_size_arg.as_str()]);
        }
        args.push(out_path.as_str());
        deps.exec_magick(args)?;
        deps.write_stdout(format!("Wrote: {out_path}\n").as_bytes())?;
        Ok(())
    }
}
