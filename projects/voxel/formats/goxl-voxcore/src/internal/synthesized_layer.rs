use crate::{GoxlExtLayer, GoxlExtPlacement, layer_provenance};
use goxl::GoxlLayer;

/// A layer entry for a node the file did not load: Goxel's default layer with
/// `id`, stamping `placements`. The synthesizer builds every entry this way,
/// and a node retained after a load gets one from the hook.
pub fn synthesized_layer(id: i32, placements: Vec<GoxlExtPlacement>) -> GoxlExtLayer {
    GoxlExtLayer {
        placements,
        ..layer_provenance(&GoxlLayer {
            id,
            ..GoxlLayer::default()
        })
    }
}
