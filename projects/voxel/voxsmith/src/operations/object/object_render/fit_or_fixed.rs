/// A length the `fit` rule can set: an orbit's distance or an orthographic
/// view's scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FitOrFixed {
    /// The length that fits the subject's bounding sphere into the shorter
    /// image axis.
    Fit,

    /// A length in meters.
    Fixed(f64),
}
