use crate::{SdfjAxis, SdfjBorder};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A pattern picking one material per cell: an entry of
/// [`SdfjFile::patterns`](crate::SdfjFile::patterns). `kind` holds the call.
/// A key the model leaves out reads `None`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase",
        deny_unknown_fields
    )
)]
pub enum SdfjPattern {
    /// Slabs across an axis that cycle through the materials.
    Bands {
        materials: Vec<usize>,

        axis: SdfjAxis,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        period: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        warp: Option<f64>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        seed: Option<f64>,
    },

    /// Courses of bricks that each take a random pick, with an optional border
    /// between them.
    Bricks {
        materials: Vec<usize>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        length: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        height: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        axis: Option<SdfjAxis>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        seed: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        border: Option<SdfjBorder>,
    },

    /// Irregular cells that each take a random pick, with an optional border
    /// between them.
    Cells {
        materials: Vec<usize>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        size: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        seed: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        border: Option<SdfjBorder>,
    },

    /// Cubes that alternate the materials.
    Checker {
        materials: Vec<usize>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        size: f64,
    },

    /// Equal spans along an axis that take the materials in order.
    Gradient {
        materials: Vec<usize>,

        axis: SdfjAxis,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        from: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        to: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        warp: Option<f64>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        seed: Option<f64>,
    },

    /// Rings around an axis that each take a random pick.
    Grain {
        materials: Vec<usize>,

        axis: SdfjAxis,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        period: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        warp: Option<f64>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        seed: f64,
    },

    /// Fractal noise split into value ranges that take the materials in order.
    Noise {
        materials: Vec<usize>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        scale: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                serialize_with = "crate::finite",
                skip_serializing_if = "Option::is_none"
            )
        )]
        octaves: Option<f64>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        seed: f64,
    },

    /// A base material with random accents.
    Speckle {
        base: usize,

        accents: Vec<usize>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        size: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        density: f64,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        seed: f64,
    },
}
