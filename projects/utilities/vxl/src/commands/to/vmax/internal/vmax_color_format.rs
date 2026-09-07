use crate::CliValue;
use voxconv::vmax::VMaxColorFormat;

impl CliValue for VMaxColorFormat {
    const VARIANTS: &'static [Self] = &[
        VMaxColorFormat::Png,
        VMaxColorFormat::Plist,
        VMaxColorFormat::All,
    ];

    fn name(self) -> &'static str {
        match self {
            VMaxColorFormat::Png => "png",
            VMaxColorFormat::Plist => "plist",
            VMaxColorFormat::All => "all",
        }
    }

    fn help(self) -> &'static str {
        match self {
            VMaxColorFormat::Png => "Store colors as a `palette*.png` image",
            VMaxColorFormat::Plist => "Store colors in the `palette*.settings.vmaxpsb` sidecar",
            VMaxColorFormat::All => "Store colors in both the image and the sidecar",
        }
    }
}
