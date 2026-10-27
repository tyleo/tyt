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
            VMaxColorFormat::Png => {
                "Store colors as a `256 x 1` `palette*.png` image and leave the \
                 `palette*.settings.vmaxpsb` sidecar without a `colors` table"
            }

            VMaxColorFormat::Plist => {
                "Store colors in the `palette*.settings.vmaxpsb` sidecar's `colors` table and \
                 write no `palette*.png`. The `pal` reference still names the absent image"
            }

            VMaxColorFormat::All => "Store colors in both the image and the sidecar",
        }
    }
}
