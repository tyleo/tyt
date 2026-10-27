use palette::LinSrgb;

/// A linear-light RGB color without alpha, backed by [`palette::LinSrgb`].
/// Components are nominally `[0, 1]` and may exceed it out of gamut.
pub type TyLinSrgb<T> = LinSrgb<T>;
