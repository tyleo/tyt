use palette::Oklaba;

/// An OKLab perceptual color with straight alpha, backed by [`palette::Oklaba`].
/// Reached from linear light with `into_color`. The axes are `l` / `a` / `b`.
pub type TyOklabColor<T> = Oklaba<T>;
