use crate::{TyCielabColor, TyOklabColor, TySrgb, TyVector3};

/// Bridges a color's three spatial channels to a [`TyVector3`] for distance
/// math, dropping alpha where present. palette cannot know `TyVector3`, so this
/// glue lives in ty-math; it reads the channels through palette's fields (the
/// alpha types reach `l` / `a` / `b` by `Deref`).
pub trait TyColorToVector3<T> {
    /// The color's three channels as a [`TyVector3`].
    fn to_vector3(&self) -> TyVector3<T>;
}

impl<T: Copy> TyColorToVector3<T> for TySrgb<T> {
    fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.red, self.green, self.blue)
    }
}

impl<T: Copy> TyColorToVector3<T> for TyOklabColor<T> {
    fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.l, self.a, self.b)
    }
}

impl<T: Copy> TyColorToVector3<T> for TyCielabColor<T> {
    fn to_vector3(&self) -> TyVector3<T> {
        TyVector3::new(self.l, self.a, self.b)
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
