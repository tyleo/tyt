use palette::LinSrgb;

/// A linear-light RGB color without alpha, backed by [`palette::LinSrgb`], the
/// three-component companion to [`TyLinSrgba`](crate::TyLinSrgba). Components are
/// nominally `[0, 1]` and may exceed it out of gamut.
pub type TyLinSrgb<T = f32> = LinSrgb<T>;
