use crate::SdfjMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The names a document gives its entries:
/// [`SdfjFile::names`](crate::SdfjFile::names). A write leaves each empty map
/// out, and a read takes a missing one as empty.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(rename_all = "camelCase", deny_unknown_fields)
)]
pub struct SdfjNames {
    /// Indices into [`SdfjFile::shapes3d`](crate::SdfjFile::shapes3d).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjMap::is_empty")
    )]
    pub shapes3d: SdfjMap<usize>,

    /// Indices into [`SdfjFile::shapes2d`](crate::SdfjFile::shapes2d).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjMap::is_empty")
    )]
    pub shapes2d: SdfjMap<usize>,

    /// Indices into [`SdfjFile::materials`](crate::SdfjFile::materials).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjMap::is_empty")
    )]
    pub materials: SdfjMap<usize>,

    /// Indices into [`SdfjFile::patterns`](crate::SdfjFile::patterns).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjMap::is_empty")
    )]
    pub patterns: SdfjMap<usize>,

    /// Indices into [`SdfjFile::steps`](crate::SdfjFile::steps).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjMap::is_empty")
    )]
    pub steps: SdfjMap<usize>,

    /// Indices into [`SdfjFile::nodes`](crate::SdfjFile::nodes): the node each
    /// part writes.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "SdfjMap::is_empty")
    )]
    pub parts: SdfjMap<usize>,
}

impl SdfjNames {
    /// Whether every map is empty.
    pub fn is_empty(&self) -> bool {
        let SdfjNames {
            shapes3d,
            shapes2d,
            materials,
            patterns,
            steps,
            parts,
        } = self;

        [shapes3d, shapes2d, materials, patterns, steps, parts]
            .iter()
            .all(|map| map.is_empty())
    }
}
