use crate::{TyCielabColorF64, TyOklabColorF64, TySrgbF64, TyVector3F64};

/// Bridges a color's three spatial channels to a [`TyVector3F64`] for distance
/// math, dropping alpha where present. palette cannot know the vector type, so
/// this glue lives in ty-math; it reads the channels through palette's fields
/// (the alpha types reach `l` / `a` / `b` by `Deref`). Only `f64` colors are used
/// for clustering, so the bridge is `f64`-only.
pub trait TyColorToVector3 {
    /// The color's three channels as a [`TyVector3F64`].
    fn to_vector3(&self) -> TyVector3F64;
}

impl TyColorToVector3 for TySrgbF64 {
    fn to_vector3(&self) -> TyVector3F64 {
        TyVector3F64::new(self.red, self.green, self.blue)
    }
}

impl TyColorToVector3 for TyOklabColorF64 {
    fn to_vector3(&self) -> TyVector3F64 {
        TyVector3F64::new(self.l, self.a, self.b)
    }
}

impl TyColorToVector3 for TyCielabColorF64 {
    fn to_vector3(&self) -> TyVector3F64 {
        TyVector3F64::new(self.l, self.a, self.b)
    }
}

#[cfg(test)]
mod tests {
    use crate::{TyColorToVector3, TyOklabColorF64, TySrgbF64, TyVector3F64};

    #[test]
    fn srgb_reads_channels() {
        // The channels map straight to a point for distance math.
        assert_eq!(
            TySrgbF64::new(0.25, 0.5, 0.75).to_vector3(),
            TyVector3F64::new(0.25, 0.5, 0.75)
        );
    }

    #[test]
    fn oklab_drops_alpha() {
        // The three axes carry through; alpha is dropped.
        assert_eq!(
            TyOklabColorF64::new(0.5, 0.1, -0.2, 1.0).to_vector3(),
            TyVector3F64::new(0.5, 0.1, -0.2)
        );
    }
}
