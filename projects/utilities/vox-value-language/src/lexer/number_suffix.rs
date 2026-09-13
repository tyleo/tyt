/// The type a literal's suffix pins.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum NumberSuffix {
    F32,
    U8,
    U16,
    U32,
}

impl NumberSuffix {
    /// The suffix a spelling names, if any.
    pub(crate) fn from_spelling(spelling: &str) -> Option<NumberSuffix> {
        match spelling {
            "f32" => Some(NumberSuffix::F32),
            "u8" => Some(NumberSuffix::U8),
            "u16" => Some(NumberSuffix::U16),
            "u32" => Some(NumberSuffix::U32),
            _ => None,
        }
    }
}
