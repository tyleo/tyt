use crate::{VoxjBackend, VoxjHierarchyNode, VoxjPalette, VoxjValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The body of a Voxel Json document, generic over the object representation
/// (see [`VoxjBackend`]).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        rename_all = "camelCase",
        bound(
            serialize = "Backend::Object: Serialize",
            deserialize = "Backend::Object: Deserialize<'de>"
        )
    )
)]
pub struct VoxjMain<Backend: VoxjBackend> {
    pub objects: Vec<Backend::Object>,

    pub palettes: Vec<VoxjPalette>,

    pub hierarchy_nodes: Vec<VoxjHierarchyNode>,

    /// Indices into [`hierarchy_nodes`](Self::hierarchy_nodes); the scene's
    /// roots.
    pub root_hierarchy_nodes: Vec<usize>,

    /// Optional namespace for user-defined extensions, conventionally
    /// vendor-keyed. The core format assigns it no meaning and guarantees
    /// nothing about its contents; consumers ignore extensions they do not
    /// recognize.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ext: Option<VoxjValue>,
}
