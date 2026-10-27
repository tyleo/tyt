use std::fmt::{Display, Formatter, Result as FmtResult};

/// The vec width.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Dimension {
    /// One component.
    Vec1,

    /// Two components.
    Vec2,

    /// Three components.
    Vec3,

    /// Four components.
    Vec4,
}

impl Dimension {
    /// The dimension with this many components, one through four.
    pub fn from_width(width: usize) -> Option<Dimension> {
        match width {
            1 => Some(Dimension::Vec1),
            2 => Some(Dimension::Vec2),
            3 => Some(Dimension::Vec3),
            4 => Some(Dimension::Vec4),
            _ => None,
        }
    }

    /// The component count.
    pub fn width(self) -> usize {
        match self {
            Dimension::Vec1 => 1,
            Dimension::Vec2 => 2,
            Dimension::Vec3 => 3,
            Dimension::Vec4 => 4,
        }
    }
}

impl Display for Dimension {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(formatter, "vec{}", self.width())
    }
}
