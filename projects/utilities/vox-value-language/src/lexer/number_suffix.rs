/// The type a literal's suffix pins.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumberSuffix {
    F64,

    U8,

    U16,

    U32,
}

impl NumberSuffix {
    /// The suffix a spelling names, if any.
    pub(crate) fn from_spelling(spelling: &str) -> Option<NumberSuffix> {
        match spelling {
            "f64" => Some(NumberSuffix::F64),
            "u8" => Some(NumberSuffix::U8),
            "u16" => Some(NumberSuffix::U16),
            "u32" => Some(NumberSuffix::U32),
            _ => None,
        }
    }
}
