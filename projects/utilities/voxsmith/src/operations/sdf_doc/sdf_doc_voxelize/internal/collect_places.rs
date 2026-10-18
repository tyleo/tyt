use crate::operations::sdf_doc::SdfPlace;
use branded_id::U32Id;
use sdfcore::{BSdfNode, SdfState};
use ty_math::{TyQuaternionExt, TyQuaternionF64, TyVector3F64};

/// Every place of every part in `state`, each before its children, from the
/// first root part. The places' grids start empty.
pub fn collect_places(state: &SdfState) -> Vec<SdfPlace> {
    let mut places = Vec::new();

    for &node_id in &state.root_node_ids {
        visit(state, node_id, None, &mut places);
    }

    places
}

/// Adds the place of the node at `node_id` under the place at `parent`, then
/// the places of its children.
fn visit(
    state: &SdfState,
    node_id: U32Id<BSdfNode>,
    parent: Option<usize>,
    places: &mut Vec<SdfPlace>,
) {
    let node = &state.nodes[node_id.to_usize_id()];

    let (mut path, parent_offset) = match parent {
        Some(parent) => (places[parent].path.clone(), places[parent].offset),
        None => (Vec::new(), TyVector3F64::ZERO),
    };
    path.push(node.name.clone());

    let offset = parent_offset + node.offset.unwrap_or(TyVector3F64::ZERO);
    let index = places.len();

    places.push(SdfPlace {
        node_id,
        parent,
        path,
        pivot: node.pivot.unwrap_or(TyVector3F64::ZERO) + offset,
        offset,
        rotation: node.rotation.iter().fold(
            TyQuaternionF64::IDENTITY,
            |rotation, axis_rotation| {
                rotation.rotate_around_axis(
                    axis_rotation.axis.unit_vector3_f64(),
                    axis_rotation.degrees.to_radians(),
                )
            },
        ),
        grid_indices: Vec::new(),
    });

    for &child_node_id in &node.child_node_ids {
        visit(state, child_node_id, Some(index), places);
    }
}
