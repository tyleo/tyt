use crate::{BSdfMaterial, SdfBorder};
use branded_id::U32Id;
use ty_math::TyAxis3;

/// A pattern picking one material per cell: an entry of
/// [`SdfState::patterns`](crate::SdfState::patterns). A field left `None`
/// takes its default.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfPattern {
    /// Slabs across an axis that cycle through the materials.
    Bands {
        material_ids: Vec<U32Id<BSdfMaterial>>,

        axis: TyAxis3,

        period: f64,

        warp: Option<f64>,

        seed: Option<f64>,
    },

    /// Irregular cells that each take a random pick, with an optional border
    /// between them.
    Cells {
        material_ids: Vec<U32Id<BSdfMaterial>>,

        size: f64,

        seed: f64,

        border: Option<SdfBorder>,
    },

    /// Cubes that alternate the materials.
    Checker {
        material_ids: Vec<U32Id<BSdfMaterial>>,

        size: f64,
    },

    /// Equal spans along an axis that take the materials in order.
    Gradient {
        material_ids: Vec<U32Id<BSdfMaterial>>,

        axis: TyAxis3,

        from: f64,

        to: f64,

        warp: Option<f64>,

        seed: Option<f64>,
    },

    /// Rings around an axis that each take a random pick.
    Grain {
        material_ids: Vec<U32Id<BSdfMaterial>>,

        axis: TyAxis3,

        period: f64,

        warp: Option<f64>,

        seed: f64,
    },

    /// Fractal noise split into value ranges that take the materials in order.
    Noise {
        material_ids: Vec<U32Id<BSdfMaterial>>,

        scale: f64,

        octaves: Option<f64>,

        seed: f64,
    },

    /// A base material with random accents.
    Speckle {
        base_id: U32Id<BSdfMaterial>,

        accent_ids: Vec<U32Id<BSdfMaterial>>,

        size: f64,

        density: f64,

        seed: f64,
    },
}
