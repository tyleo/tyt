use crate::{Dimension, Domain, Scalar};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A value's type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Type {
    /// What the value has one entry per.
    pub domain: Domain,

    /// The vec width.
    pub dimension: Dimension,

    /// The component type.
    pub scalar: Scalar,
}

impl Display for Type {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        write!(
            formatter,
            "{} {} {}",
            self.domain, self.dimension, self.scalar
        )
    }
}
