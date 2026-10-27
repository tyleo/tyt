use crate::BSdfMaterial;
use branded_id::U32Id;

/// One `shades` call: an entry of
/// [`SdfState::shades`](crate::SdfState::shades). Each shade the call returns
/// sits in `materials` as a [`Shade`](crate::SdfMaterial::Shade) referencing
/// this entry.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfShades {
    /// The material the shades vary.
    pub base_id: U32Id<BSdfMaterial>,

    /// How many shades the call returns.
    pub count: f64,

    /// The lightness step between neighboring shades.
    pub spread: Option<f64>,
}
