use std::fmt::{Display, Formatter, Result as FmtResult};

/// The transfer a written value declares.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transfer {
    /// Applies no transfer.
    Linear,

    /// Applies the sRGB transfer.
    Srgb,
}

impl Display for Transfer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            Transfer::Linear => "linear",
            Transfer::Srgb => "srgb",
        })
    }
}
