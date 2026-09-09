use crate::ext::GoxlExt;
use std::any::Any;
use voxcore::{
    VoxMap,
    ext::{FollowListing, Result, VoxExt, encode_entry},
};

/// The Goxel ext as a state's ext. Its block is the `goxl` entry. The layers
/// follow the hierarchy listing. A retained node takes no entry. The writer
/// fills it in like a synthesized layer. A layer's block indices follow the
/// object listing.
impl VoxExt for GoxlExt {
    fn to_vox_ext(&self) -> Result<VoxMap> {
        encode_entry(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn VoxExt> {
        Box::new(self.clone())
    }

    fn hierarchy_node_did_retain(&mut self, index: usize) {
        self.layers.follow_retain(index);
    }

    fn hierarchy_node_will_release(&mut self, index: usize) {
        self.layers.follow_release(index);
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
