use crate::{
    QbExt, QbExtMatrix, QbVoxMain, Result, SOLID_VISIBILITY, duplicate_object, qb_placements,
};
use branded_id::U32Id;
use qbcl::qb::QbFile;
use std::collections::{HashMap, HashSet};
use ty_math::{TyTransformF64, TyVector3I32};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain};

/// Gives a bare state a synthesized [`QbExt`], the state
/// [`to_qb_file`](crate::to_qb_file) writes as a file synthesized from the
/// scene. Qubicle Binary has a flat matrix list and no hierarchy, so the
/// scene flattens first: each object placement becomes one root node placing
/// one object at the placement's world translation, summed down from the
/// roots and rounded to whole voxels. An object placed several times is
/// duplicated per extra placement. An object no node places gets a root at
/// the origin, after the placed ones. The objects take placement order. Each
/// then takes an entry with the placement's name and position and the plain
/// solid visibility byte per voxel. The header takes the defaults: `RGBA`,
/// left-handed, uncompressed, with plain visibility bytes.
///
/// Lossy where Qubicle Binary cannot represent the source: grouping
/// collapses, node rotation and scale drop, and a color's alpha drops
/// because a Qubicle voxel stores none.
pub fn to_qb_vox_main(mut state: VoxMain<()>) -> Result<QbVoxMain> {
    let placements = qb_placements(&state);

    // Every placement gets an object of its own: the object itself the first
    // time, a duplicate after.
    let mut taken: HashSet<U32Id<BVoxObject>> = HashSet::new();
    let mut object_ids = Vec::with_capacity(placements.len());
    for placement in &placements {
        if taken.insert(placement.object_id) {
            object_ids.push(placement.object_id);
            continue;
        }

        let object = state
            .object(placement.object_id)
            .expect("a placement's object is one of the state's");
        object_ids.push(state.retain_object(duplicate_object(object))?);
    }

    for (index, &object_id) in object_ids.iter().enumerate() {
        state.move_object(object_id, index)?;
    }

    let node_ids: Vec<U32Id<BVoxHierarchyNode>> = state
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .collect();
    state.set_root_hierarchy_node_ids(Vec::new())?;
    for &node_id in &node_ids {
        let mut node = state
            .hierarchy_node(node_id)
            .expect("a listed node")
            .clone();
        node.child_node_ids.clear();
        state.set_hierarchy_node(node_id, node)?;
    }
    for node_id in node_ids {
        state.release_hierarchy_node(node_id)?;
    }

    let mut root_ids = Vec::with_capacity(placements.len());
    for (placement, &object_id) in placements.iter().zip(&object_ids) {
        root_ids.push(state.retain_hierarchy_node(VoxHierarchyNode {
            name: placement.name.clone(),
            transform: TyTransformF64::from_translation(
                TyVector3I32::from_array(placement.position).as_dvec3(),
            ),
            child_node_ids: Vec::new(),
            child_object_ids: vec![object_id],
        })?);
    }
    state.set_root_hierarchy_node_ids(root_ids)?;

    let live_counts: HashMap<U32Id<BVoxObject>, usize> = state
        .iter_objects()
        .map(|(object_id, object)| (object_id, object.live_count()))
        .collect();
    let matrices = placements
        .iter()
        .zip(&object_ids)
        .map(|(placement, object_id)| {
            Some(QbExtMatrix {
                name: placement.name.clone(),
                position: placement.position,
                visibility: vec![SOLID_VISIBILITY; live_counts[object_id]],
            })
        })
        .collect();

    let file = QbFile::default();
    let ext = QbExt {
        version: file.version,
        bgra: false,
        right_handed: false,
        compressed: file.compressed,
        visibility_mask_encoded: file.visibility_mask_encoded,
        matrices,
    };

    Ok(state.put_ext(ext))
}

#[cfg(test)]
mod tests {
    use crate::{SOLID_VISIBILITY, from_qb_file, to_qb_file, to_qb_vox_main};
    use branded_id::U32Id;
    use qbcl::qb::QbFile;
    use std::collections::BTreeSet;
    use ty_math::{TyHexColor, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject,
        VoxPalette, VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
    };

    /// The linear-light components of a `#RRGGBB` hex string.
    fn linear_rgb(hex: &str) -> [f64; 3] {
        let linear =
            lin_srgba_f64_from_srgba_u8(TySrgbaU8::from_hex(hex).expect("a valid hex color"));
        [linear.red, linear.green, linear.blue]
    }

    /// A bare state built straight from voxcore: a red-green object and a
    /// blue object sharing one `baseColor` palette, placed by a hierarchy of
    /// a nested group and two roots. This is the cross-format synthesis input.
    fn source_state() -> VoxMain<()> {
        let mut state = VoxMain::default();

        // One baseColor palette: red, green, blue.
        let value_pool_id = state.retain_value_pool(
            VoxValuePool::vec_3_float(
                ["#FF0000", "#00FF00", "#0000FF"]
                    .iter()
                    .map(|hex| linear_rgb(hex))
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        for index in 0..3 {
            palette
                .retain_material(vec![U32Id::from_u32(index)])
                .expect("one value id for the one property");
        }
        let palette_id = state.retain_palette(palette).unwrap();
        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red then a green voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.retain_layer(palette_id, material_id(0));
        for (x, material_index) in [(0u32, 0u32), (1, 1)] {
            let voxel_id = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel_id, &[material_id(material_index)])
                .expect("one sample for the one layer");
        }
        state.retain_object(wide).unwrap();

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.retain_layer(palette_id, material_id(0));
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(2)])
            .expect("one sample for the one layer");
        state.retain_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at =
            |x: f64, y: f64, z: f64| TyTransformF64::from_translation(TyVector3F64::new(x, y, z));

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state
            .retain_hierarchy_nodes(vec![
                VoxHierarchyNode {
                    name: "group".to_owned(),
                    child_node_ids: vec![node_id(1)],
                    child_object_ids: Vec::new(),
                    transform: TyTransformF64::default(),
                },
                VoxHierarchyNode {
                    name: "wide".to_owned(),
                    child_node_ids: Vec::new(),
                    child_object_ids: vec![object_id(0)],
                    transform: placed_at(5.0, 0.0, 0.0),
                },
                VoxHierarchyNode {
                    name: "unit".to_owned(),
                    child_node_ids: Vec::new(),
                    child_object_ids: vec![object_id(1)],
                    transform: placed_at(0.0, 3.0, 0.0),
                },
            ])
            .unwrap();
        state
            .set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        state.validate().expect("a well-formed source state");
        state
    }

    /// A solid voxel in world space: `x`, `y`, `z`, and an `rgb` color.
    type WorldVoxel = (i32, i32, i32, (u8, u8, u8));

    /// The world voxels a file places: each matrix's solid cells decoded to
    /// world coordinates and color, order-independent. Each cell's grid
    /// coordinate decodes from the storage index
    /// `x + size_x * (y + size_y * z)`. The matrix position offsets it.
    fn world_voxels(file: &QbFile) -> BTreeSet<WorldVoxel> {
        let mut set = BTreeSet::new();
        for matrix in &file.matrices {
            let [size_x, size_y, _] = matrix.size;
            for (index, voxel) in matrix.voxels.iter().enumerate() {
                if voxel.is_empty() {
                    continue;
                }
                let index = index as u32;
                let x = index % size_x;
                let yz_index = index / size_x;
                let y = yz_index % size_y;
                let z = yz_index / size_y;
                set.insert((
                    matrix.position[0] + x as i32,
                    matrix.position[1] + y as i32,
                    matrix.position[2] + z as i32,
                    (voxel.r, voxel.g, voxel.b),
                ));
            }
        }
        set
    }

    /// A default state has no objects, so the writer synthesizes an empty
    /// file.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = to_qb_vox_main(VoxMain::default()).unwrap();
        let file = to_qb_file(&state).unwrap();
        assert!(file.matrices.is_empty());
    }

    /// A bare state, such as one cross-loaded from another format, synthesizes
    /// a file: the hierarchy flattens to matrices whose world voxels and colors
    /// match the source, one matrix per placement named for its node, and the
    /// file reads back into a valid state.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let state = to_qb_vox_main(source_state()).unwrap();
        let file = to_qb_file(&state).unwrap();

        let red = (0xFF, 0, 0);
        let green = (0, 0xFF, 0);
        let blue = (0, 0, 0xFF);

        // Object 0 is placed at +5x under a group, object 1 at +3y.
        assert_eq!(
            world_voxels(&file),
            BTreeSet::from([(5, 0, 0, red), (6, 0, 0, green), (0, 3, 0, blue)])
        );

        assert_eq!(file.matrices.len(), 2);
        assert_eq!(file.matrices[0].name, "wide");
        assert_eq!(file.matrices[1].name, "unit");

        let reloaded = from_qb_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// The ext holds one entry per matrix the file will hold, and the scene
    /// flattens to match: one root per placement at its world position.
    #[test]
    fn synthesizes_an_entry_per_placement() {
        let state = to_qb_vox_main(source_state()).unwrap();

        let ext = state.ext();

        assert!(!ext.visibility_mask_encoded);

        assert_eq!(ext.matrices.len(), 2);

        let wide = ext.matrices[0].as_ref().unwrap();

        assert_eq!(wide.name, "wide");

        assert_eq!(wide.position, [5, 0, 0]);

        assert_eq!(wide.visibility, vec![SOLID_VISIBILITY; 2]);

        let unit = ext.matrices[1].as_ref().unwrap();

        assert_eq!(unit.name, "unit");

        assert_eq!(unit.position, [0, 3, 0]);

        assert_eq!(state.hierarchy_node_count(), 2);

        assert_eq!(state.root_hierarchy_node_ids().len(), 2);

        let root = state
            .hierarchy_node(state.root_hierarchy_node_ids()[0])
            .unwrap();

        assert_eq!(root.name, "wide");

        assert_eq!(
            root.transform,
            TyTransformF64::from_translation(TyVector3F64::new(5.0, 0.0, 0.0))
        );
    }

    /// An object placed twice is duplicated so each placement has a matrix,
    /// and the copy keeps the voxels.
    #[test]
    fn duplicates_an_object_placed_twice() {
        let mut state = source_state();

        let node_id = U32Id::<BVoxHierarchyNode>::from_u32(2);

        let mut node = state.hierarchy_node(node_id).unwrap().clone();

        node.child_object_ids.push(U32Id::from_u32(0));

        state.set_hierarchy_node(node_id, node).unwrap();

        let state = to_qb_vox_main(state).unwrap();

        assert_eq!(state.object_count(), 3);

        let ext = state.ext();

        assert_eq!(ext.matrices.len(), 3);

        assert_eq!(ext.matrices[2].as_ref().unwrap().position, [0, 3, 0]);

        let copy = state.object(U32Id::<BVoxObject>::from_u32(2)).unwrap();

        assert_eq!(copy.live_count(), 2);

        let file = to_qb_file(&state).unwrap();

        assert_eq!(file.matrices.len(), 3);
    }
}
