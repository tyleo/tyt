use crate::Placement;
use vmax::VMaxGroup;

/// The content box `(center, half)` is the group's derived subtree box,
/// written as the symmetric `e_c`, `e_mi`, and `e_ma` Voxel Max stores.
pub(crate) fn group_from_node(
    placement: &Placement<'_>,
    rotation: [f64; 4],
    center: [f64; 3],
    half: [f64; 3],
) -> VMaxGroup {
    let node = placement.node;
    let ext_node = placement.ext;
    VMaxGroup {
        name: node.name.clone(),
        id: ext_node.id.clone(),
        parent_id: placement.parent_id.clone(),
        hidden: None,
        position: node.transform.position.to_array(),
        rotation,
        scale: node.transform.scale.to_array(),
        ind: ext_node.index,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone(),
        t_pa: ext_node.pivot_align.clone(),
        t_pf: ext_node.pivot_face.clone(),
        t_po: None,
        center,
        bounds_min: Some([-half[0], -half[1], -half[2]]),
        bounds_max: Some(half),
    }
}
