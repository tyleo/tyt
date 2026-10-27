use std::fmt::{Display, Formatter, Result as FmtResult};

/// The part of a palette edit an error rose from.
#[derive(Clone, Debug, PartialEq)]
pub enum PaletteEditElement {
    /// The program the writes read.
    Program,

    /// The write to `property`.
    Write { property: String },
}

impl Display for PaletteEditElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PaletteEditElement::Program => f.write_str("the program"),

            PaletteEditElement::Write { property } => write!(f, "the write to `{property}`"),
        }
    }
}
