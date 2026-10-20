use crate::SdfjSide;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One step of a part's list: an entry of
/// [`SdfjFile::steps`](crate::SdfjFile::steps). `kind` holds the call. A key
/// the model leaves out reads `None`. A step that writes a material holds
/// exactly one of `material` and `pattern`.
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
pub enum SdfjStep {
    /// Fills the shape's cells.
    Add {
        name: String,

        shape: usize,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        material: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pattern: Option<usize>,
    },

    /// Empties the shape's cells.
    Carve { name: String, shape: usize },

    /// Recolors the live cells whose point `depth` meters toward a listed side
    /// lies outside the part's earlier shapes.
    Coat {
        name: String,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        material: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pattern: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        sides: Option<Vec<SdfjSide>>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        depth: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        within: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        onto: Option<Vec<usize>>,
    },

    /// Recolors the live cells inside the shape.
    Paint {
        name: String,

        shape: usize,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        material: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pattern: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        onto: Option<Vec<usize>>,
    },

    /// Fills a cube `size` wide around each point.
    Set {
        name: String,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        points: Vec<[f64; 3]>,

        #[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))]
        size: f64,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        material: Option<usize>,

        #[cfg_attr(
            feature = "serde",
            serde(
                default,
                deserialize_with = "crate::present",
                skip_serializing_if = "Option::is_none"
            )
        )]
        pattern: Option<usize>,
    },
}
