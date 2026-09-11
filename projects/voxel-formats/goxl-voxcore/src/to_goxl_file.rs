use crate::{GoxlExtSource, Result};
use goxl::GoxlFile;
use voxcore::VoxMain;

/// Writes a bare [`VoxMain`] to a Goxel [`GoxlFile`] synthesized from its
/// scene, the inverse of [`from_goxl_file`](crate::from_goxl_file). Goxel has
/// flat layers of placed `16 x 16 x 16` blocks and no hierarchy, so each object
/// placement becomes one layer at its world translation rounded to whole
/// voxels. Grouping, rotation, and scale drop.
/// [`ext::to_goxl_file_with_ext`](crate::ext::to_goxl_file_with_ext) writes a
/// loaded file back exactly.
///
/// Errors when an object's `baseColor` draws from a non-color value pool.
pub fn to_goxl_file(state: &VoxMain<()>) -> Result<GoxlFile> {
    GoxlExtSource::write_goxl(state)
}

#[cfg(test)]
mod tests {
    use crate::{from_goxl_file, to_goxl_file};
    use branded_id::U32Id;
    use goxl::{GoxlBlock, GoxlFile};
    use std::collections::BTreeSet;
    use ty_math::{
        TyHexColor, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32,
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject,
        VoxPalette, VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
    };

    /// The linear-light components of a `#RRGGBBAA` hex string.
    fn linear_rgba(hex: &str) -> [f64; 4] {
        lin_srgba_f64_from_srgba_u8(TySrgbaU8::from_hex(hex).expect("a valid hex color")).into()
    }

    /// A bare state built straight from voxcore: a red-green object and a
    /// blue object sharing one `rgba` palette, placed by a hierarchy of a
    /// nested group and two roots. This is the cross-format synthesis input.
    fn source_state() -> VoxMain<()> {
        let mut state = VoxMain::default();

        // One baseColor palette: a transparent placeholder, then red,
        // green, blue.
        let value_pool_id = state.retain_value_pool(
            VoxValuePool::vec_4_float(
                ["#00000000", "#FF0000FF", "#00FF00FF", "#0000FFFF"]
                    .iter()
                    .map(|hex| linear_rgba(hex))
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        for index in 0..4 {
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
        for (x, material_index) in [(0u32, 1u32), (1, 2)] {
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
        unit.retain_voxel(voxel_id, &[material_id(3)])
            .expect("one sample for the one layer");
        state.retain_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at = |x: f64, y: f64, z: f64| {
            TyTransformF64::new(
                TyVector3F64::new(x, y, z),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::new(1.0, 1.0, 1.0),
            )
        };

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

    /// A solid voxel in world space: `x`, `y`, `z`, and an `rgba` color.
    type WorldVoxel = (i32, i32, i32, (u8, u8, u8, u8));

    /// The world voxels a file places: each layer block's solid cells decoded
    /// to world coordinates and color, order-independent.
    fn world_voxels(file: &GoxlFile) -> BTreeSet<WorldVoxel> {
        let stride = GoxlBlock::SIZE as usize;
        let mut set = BTreeSet::new();
        for layer in &file.layers {
            for placement in &layer.blocks {
                let block = &file.blocks[placement.block_index as usize];
                for (index, voxel) in block.voxels.iter().enumerate() {
                    if voxel.is_empty() {
                        continue;
                    }
                    let x = index % stride;
                    let y = index / stride % stride;
                    let z = index / (stride * stride);
                    set.insert((
                        placement.position[0] + x as i32,
                        placement.position[1] + y as i32,
                        placement.position[2] + z as i32,
                        (voxel.r, voxel.g, voxel.b, voxel.a),
                    ));
                }
            }
        }
        set
    }

    /// A default state has no objects, so the writer synthesizes an empty
    /// file.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = VoxMain::default();
        let file = to_goxl_file(&state).unwrap();
        assert!(file.blocks.is_empty());
        assert!(file.layers.is_empty());
    }

    /// A bare state, such as one cross-loaded from another format, synthesizes
    /// a file: the hierarchy flattens to layers of placed blocks whose world
    /// voxels and colors match the source, one layer per placement named for
    /// its node, and the file reads back into a valid state.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let state = source_state();
        let file = to_goxl_file(&state).unwrap();

        let red = (0xFF, 0, 0, 0xFF);
        let green = (0, 0xFF, 0, 0xFF);
        let blue = (0, 0, 0xFF, 0xFF);

        // Object 0 is placed at +5x under a group, object 1 at +3y.
        assert_eq!(
            world_voxels(&file),
            BTreeSet::from([(5, 0, 0, red), (6, 0, 0, green), (0, 3, 0, blue)])
        );

        assert_eq!(file.layers.len(), 2);
        assert_eq!(file.layers[0].name, "wide");
        assert_eq!(file.layers[1].name, "unit");

        // Each object tiles to one block, and each block reads back as its own
        // object in a valid state.
        assert_eq!(file.blocks.len(), 2);
        let reloaded = from_goxl_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }
}
