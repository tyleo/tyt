use crate::CliValue;
use voxsmith::operations::mesh::Transfer;

impl CliValue for Transfer {
    const VARIANTS: &'static [Self] = &[Transfer::Linear, Transfer::Srgb];

    fn name(self) -> &'static str {
        match self {
            Transfer::Linear => "linear",
            Transfer::Srgb => "srgb",
        }
    }

    fn help(self) -> &'static str {
        match self {
            Transfer::Linear => "No transfer",
            Transfer::Srgb => "The sRGB transfer, for an image a viewer reads as color",
        }
    }
}
