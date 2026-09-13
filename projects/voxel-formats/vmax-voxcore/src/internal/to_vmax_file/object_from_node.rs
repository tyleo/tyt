use crate::{ObjectPlacement, VMaxExtNode, decode_axis_angle, unbake_position};
use vmax::VMaxObject;
use voxcore::VoxHierarchyNode;

#[allow(clippy::too_many_arguments)]
pub(crate) fn object_from_node(
    node: &VoxHierarchyNode,
    ext_node: &VMaxExtNode,
    parent_id: Option<String>,
    rotation: [f64; 4],
    placement: &ObjectPlacement,
    data: String,
    pal: String,
    suffix: &str,
) -> VMaxObject {
    VMaxObject {
        name: node.name.clone(),
        data,
        palette: pal,
        history: format!("history{suffix}.vmaxhb"),
        id: ext_node.id.clone(),
        parent_id,
        hidden: None,
        position: unbake_position(&node.transform, decode_axis_angle(rotation), placement),
        rotation,
        scale: node.transform.scale.to_array(),
        ind: ext_node.index,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone(),
        t_pa: ext_node.pivot_align.clone(),
        t_pf: ext_node.pivot_face.clone(),
        t_po: None,
        center: placement.center,
        bounds_min: Some(placement.bounds_min),
        bounds_max: Some(placement.bounds_max),
    }
}
