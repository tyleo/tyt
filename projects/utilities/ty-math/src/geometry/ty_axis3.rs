use crate::TyVector3F64;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// One axis of a 3D vector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TyAxis3 {
    /// The x axis.
    X,

    /// The y axis.
    Y,

    /// The z axis.
    Z,
}

impl TyAxis3 {
    /// The axis's component index into a vector, `0` through `2`.
    pub fn index(self) -> usize {
        match self {
            TyAxis3::X => 0,
            TyAxis3::Y => 1,
            TyAxis3::Z => 2,
        }
    }

    /// The unit vector along the axis.
    pub fn unit_vector3_f64(self) -> TyVector3F64 {
        match self {
            TyAxis3::X => TyVector3F64::X,
            TyAxis3::Y => TyVector3F64::Y,
            TyAxis3::Z => TyVector3F64::Z,
        }
    }
}

impl Display for TyAxis3 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let name = match self {
            TyAxis3::X => "x",
            TyAxis3::Y => "y",
            TyAxis3::Z => "z",
        };

        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyAxis3, TyVector3F64, TyVector3U32};

    #[test]
    fn indexes_a_vector_and_prints_lowercase() {
        let vector = TyVector3U32::new(4, 5, 6);

        assert_eq!(vector[TyAxis3::Y.index()], 5);
        assert_eq!(TyAxis3::Z.to_string(), "z");
    }

    #[test]
    fn a_unit_vector_runs_along_its_axis() {
        assert_eq!(TyAxis3::Y.unit_vector3_f64(), TyVector3F64::Y);
    }
}
