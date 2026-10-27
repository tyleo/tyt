use crate::{Dimension, Domain, Scalar, Type};

/// The type of the given domain, dimension, and scalar.
pub fn ty(domain: Domain, dimension: Dimension, scalar: Scalar) -> Type {
    Type {
        domain,
        dimension,
        scalar,
    }
}
