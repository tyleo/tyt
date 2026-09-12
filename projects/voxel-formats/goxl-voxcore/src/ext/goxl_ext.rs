use crate::{
    GoxlExtCamera, GoxlExtImage, GoxlExtLayer, GoxlExtLight, GoxlExtMaterial, GoxlExtPreview,
    GoxlExtUnknownChunk,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use voxcore::VoxExt;

/// The `goxl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Goxel `.gox` state with no native voxcore home, kept so a file loaded from a
/// Goxel package can be written back exactly.
///
/// The shared `BL16` voxel blocks become native objects and the per-layer block
/// placements that stamp them become the hierarchy nodes; this holds the rest,
/// with the per-layer entries aligned by index with the hierarchy nodes so the
/// file rebuilds exactly. The layers follow the state through the
/// [`VoxExt`](voxcore::VoxExt) hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GoxlExt {
    /// The format version from the header.
    pub version: i32,

    /// The `IMG ` image metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub image: GoxlExtImage,

    /// The `PREV` preview thumbnail, or `None` when the file omits it.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub preview: Option<GoxlExtPreview>,

    /// The `MATE` materials, in stored order; a layer names one by index.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub materials: Vec<GoxlExtMaterial>,

    /// Per-layer provenance, aligned by index with the hierarchy nodes. A node
    /// retained after the load has `None`. The writer fills it in like a
    /// synthesized layer.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub layers: Vec<Option<GoxlExtLayer>>,

    /// The `CAMR` cameras, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub cameras: Vec<GoxlExtCamera>,

    /// The `LIGH` light settings, or `None` when the file omits them.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub light: Option<GoxlExtLight>,

    /// Chunks the goxl crate does not model, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "unknown-chunks",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub unknown_chunks: Vec<GoxlExtUnknownChunk>,
}

/// The Goxel ext as a state's ext. The layers follow the hierarchy listing.
/// A retained node takes no entry. The writer fills it in like a synthesized
/// layer. A layer's block indices follow the object listing.
impl VoxExt for GoxlExt {
    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.layers.insert(index, None);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        self.layers.remove(index);
    }

    fn object_will_release(&mut self, index: usize) {
        // An index past `i32` matches no placement and shifts none.
        let Ok(index) = i32::try_from(index) else {
            return;
        };
        for layer in self.layers.iter_mut().flatten() {
            layer.placements.retain(|(block, _)| *block != index);
            for (block, _) in &mut layer.placements {
                if *block > index {
                    *block -= 1;
                }
            }
        }
    }

    fn object_did_move(&mut self, from: usize, to: usize) {
        // An index past `i32` reaches no placement.
        let (Ok(from), Ok(to)) = (i32::try_from(from), i32::try_from(to)) else {
            return;
        };
        for layer in self.layers.iter_mut().flatten() {
            for (block, _) in &mut layer.placements {
                *block = moved_index(*block, from, to);
            }
        }
    }
}

/// Where block index `index` lands after the entry at `from` moves to `to`.
fn moved_index(index: i32, from: i32, to: i32) -> i32 {
    if index == from {
        to
    } else if from < index && index <= to {
        index - 1
    } else if to <= index && index < from {
        index + 1
    } else {
        index
    }
}
