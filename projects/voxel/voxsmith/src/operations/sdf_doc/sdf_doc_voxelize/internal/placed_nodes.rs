use crate::{
    Error, Result,
    operations::sdf_doc::{PlaceNode, SdfPlace, meters, node_transform, whole_voxels},
    utilities::VoxelFrame,
};
use branded_id::U32Id;
use sdfcore::BSdfNode;
use std::collections::HashMap;
use ty_math::{TyVector3F64, TyVector3I32};

/// The node of each part of `places`, sampled under `frame` on the lattice of
/// cells `voxel_size` across. A part writes one node for all its places, under
/// each parent's node. `objects_at` gets a part's first place index and its
/// offset in voxels from the lattice, and returns the part's object indices.
/// Errors on a part whose places cannot share one node, and on any
/// `objects_at` error.
pub fn placed_nodes(
    places: &[SdfPlace],
    frame: VoxelFrame,
    voxel_size: f64,
    mut objects_at: impl FnMut(usize, TyVector3F64) -> Result<Vec<usize>>,
) -> Result<Vec<PlaceNode>> {
    let mut nodes: Vec<PlaceNode> = Vec::new();

    // The index in `nodes` of each part's node beside the index of the part's
    // first place.
    let mut part_nodes: HashMap<U32Id<BSdfNode>, (usize, usize)> = HashMap::new();

    // The index in `nodes` of each place's node, and whether the place wrote it.
    let mut place_nodes: Vec<(usize, bool)> = Vec::with_capacity(places.len());

    for (index, place) in places.iter().enumerate() {
        if let Some(&(node_index, first)) = part_nodes.get(&place.node_id) {
            check_shared_place(places, frame, voxel_size, first, index)?;
            place_nodes.push((node_index, false));
            add_child(&mut nodes, &place_nodes, place.parent, node_index);
            continue;
        }

        let shift = match frame {
            VoxelFrame::World => place.offset,
            VoxelFrame::Local => TyVector3F64::ZERO,
        };
        let lattice_offset = -(place.pivot - place.offset + shift) / voxel_size;
        let object_indices = objects_at(index, lattice_offset)?;

        let node_index = nodes.len();
        part_nodes.insert(place.node_id, (node_index, index));
        place_nodes.push((node_index, true));

        nodes.push(PlaceNode {
            name: place.path.last().expect("a place has a part").clone(),
            root: place.parent.is_none(),
            child_indices: Vec::new(),
            transform: node_transform(places, voxel_size, place),
            object_indices,
            lattice_offset,
        });
        add_child(&mut nodes, &place_nodes, place.parent, node_index);
    }

    Ok(nodes)
}

/// Errors unless the place at `index` can share the node of its part's first
/// place at `first`. One node takes one position from its parent's pivot, and
/// under the world frame one object holds the cells of both places.
fn check_shared_place(
    places: &[SdfPlace],
    frame: VoxelFrame,
    voxel_size: f64,
    first: usize,
    index: usize,
) -> Result<()> {
    let first_path = places[first].path.join("/");
    let fail = |message: String| {
        Error::invalid(format!(
            "{}: part must sit {message}",
            places[index].path.join("/")
        ))
    };

    // A root part sits in one place, so every later place sits under a part.
    let spot = |index: usize| {
        let place = &places[index];
        let parent = place
            .parent
            .expect("a part in several places sits under parts");

        place.pivot - places[parent].pivot
    };
    let (first_spot, spot) = (spot(first), spot(index));

    if whole_voxels((spot - first_spot) / voxel_size) != Some(TyVector3I32::ZERO) {
        return Err(fail(format!(
            "{} from its parent's pivot as at {first_path}, not {}; give its parents one pivot \
             or build a fresh part for each place",
            listed(first_spot),
            listed(spot),
        )));
    }

    if frame == VoxelFrame::World {
        let voxels = (places[index].offset - places[first].offset) / voxel_size;

        if whole_voxels(voxels).is_none() {
            return Err(fail(format!(
                "a whole number of voxels from {first_path}, not {} voxels; give it \
                 whole-voxel offsets, sample under the local frame, or build a fresh part for \
                 each place",
                listed(voxels),
            )));
        }
    }

    Ok(())
}

/// `value` as an error message shows it.
fn listed(value: TyVector3F64) -> String {
    format!(
        "[{}, {}, {}]",
        meters(value.x),
        meters(value.y),
        meters(value.z)
    )
}

/// Lists the node at `node_index` under the node of the place at `parent` when
/// that place wrote its node. Every place of a part lists the same children.
fn add_child(
    nodes: &mut [PlaceNode],
    place_nodes: &[(usize, bool)],
    parent: Option<usize>,
    node_index: usize,
) {
    if let Some(parent) = parent
        && let (parent_index, true) = place_nodes[parent]
    {
        nodes[parent_index].child_indices.push(node_index);
    }
}
