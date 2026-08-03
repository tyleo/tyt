use palette::{Laba, white_point::D65};

/// A CIELAB perceptual color with straight alpha under the D65 white point,
/// backed by [`palette::Laba`]. The alias pins `D65` because palette orders the
/// white point ahead of the component; reached from linear light with
/// `into_color`.
pub type TyCielabColor<T> = Laba<D65, T>;
