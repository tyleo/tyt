use crate::{
    Result,
    operations::sdf_doc::{
        ObjectCells, PlaceNode, SdfMaterialProperties, WHOLE_TOLERANCE, retain_sdf_palette,
    },
};
use branded_id::{IdVec, U32Id};
use sdfcore::BSdfMaterial;
use std::collections::HashSet;
use ty_math::{TyTransformF64, TyVector3F64};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};

/// The voxj document of `objects` under `nodes`, with a palette of the
/// `materials` the objects hold. Errors when the document rejects an object,
/// a node, or the palette.
pub fn assemble_vox_main(
    objects: Vec<ObjectCells>,
    nodes: &[PlaceNode],
    materials: &IdVec<BSdfMaterial, SdfMaterialProperties>,
) -> Result<VoxMain> {
    let mut first_met = Vec::new();
    let mut met = HashSet::new();

    for object in &objects {
        for (_, material_id) in &object.cells {
            if met.insert(*material_id) {
                first_met.push(*material_id);
            }
        }
    }

    let mut main = VoxMain::default();
    let (palette_id, palette_materials) = retain_sdf_palette(&mut main, materials, &first_met)?;

    let bases: Vec<_> = objects.iter().map(|object| object.base).collect();
    let mut object_ids = Vec::with_capacity(objects.len());

    for object in objects {
        let mut vox_object = VoxObject::new(object.name, object.size)
            .expect("a part's grid fits voxcore's cell limit");
        vox_object.set_origin(object.min + object.base);
        vox_object
            .retain_layer(palette_id)
            .expect("a fresh object has no live voxel");

        for (position, material_id) in object.cells {
            let voxel_id = VoxObject::raster_id(object.size, position)
                .expect("a live cell lies inside its object's box");

            vox_object
                .retain_voxel(voxel_id, &[palette_materials[&material_id]])
                .expect("a raster index is a voxel of the object sampling its one layer");
        }

        object_ids.push(main.retain_object(vox_object)?);
    }

    let mut node_ids: Vec<U32Id<BVoxHierarchyNode>> = Vec::with_capacity(nodes.len());
    let mut children = Vec::with_capacity(nodes.len());

    for node in nodes {
        let node_id = main.retain_hierarchy_node(VoxHierarchyNode {
            name: node.name.clone(),
            transform: node.transform,
            ..VoxHierarchyNode::default()
        })?;
        node_ids.push(node_id);

        let mut child_object_ids = Vec::new();
        let mut voxels: Vec<(TyVector3F64, Vec<U32Id<BVoxObject>>)> = Vec::new();

        for &object_index in &node.object_indices {
            let residual = snap(node.lattice_offset - bases[object_index].as_dvec3());

            if residual == TyVector3F64::ZERO {
                child_object_ids.push(object_ids[object_index]);
            } else {
                match voxels.iter_mut().find(|(known, _)| *known == residual) {
                    Some((_, object_ids_at)) => object_ids_at.push(object_ids[object_index]),
                    None => voxels.push((residual, vec![object_ids[object_index]])),
                }
            }
        }

        let mut child_node_ids = Vec::new();

        for (residual, object_ids_at) in voxels {
            let voxels_node_id = main.retain_hierarchy_node(VoxHierarchyNode {
                name: "voxels".to_owned(),
                transform: TyTransformF64 {
                    position: residual,
                    ..TyTransformF64::IDENTITY
                },
                child_object_ids: object_ids_at,
                ..VoxHierarchyNode::default()
            })?;
            child_node_ids.push(voxels_node_id);
        }

        children.push((child_node_ids, child_object_ids));
    }

    for (index, (mut child_node_ids, child_object_ids)) in children.into_iter().enumerate() {
        child_node_ids.extend(
            nodes[index]
                .child_indices
                .iter()
                .map(|&child_index| node_ids[child_index]),
        );

        main.set_hierarchy_node_children(node_ids[index], child_node_ids, child_object_ids)?;

        if nodes[index].root {
            main.push_root_hierarchy_node_id(node_ids[index])?;
        }
    }

    Ok(main)
}

/// `offset` with each component within [`WHOLE_TOLERANCE`] of zero read as
/// zero.
fn snap(offset: TyVector3F64) -> TyVector3F64 {
    TyVector3F64::from_array(offset.to_array().map(|component| {
        if component.abs() <= WHOLE_TOLERANCE {
            0.0
        } else {
            component
        }
    }))
}
