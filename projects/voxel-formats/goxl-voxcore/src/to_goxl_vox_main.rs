use crate::{
    GoxlExtPlacement, GoxlVoxMain, Result, goxl_ext_from_file, next_layer_id, synthesized_layer,
};
use branded_id::U32Id;
use goxl::{GoxlBlock, GoxlFile};
use std::collections::{BTreeMap, HashSet};
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};

/// Gives a bare main a synthesized [`GoxlExt`](crate::GoxlExt), the main
/// [`to_goxl_file`](crate::to_goxl_file) writes as a file synthesized from
/// the scene. Goxel has flat layers of placed `16 x 16 x 16` blocks and no
/// hierarchy, so the scene takes the shape a loaded file has. Every object
/// placement becomes one root node with no transform, named for the placing
/// node, holding the placement's tiles: `16 x 16 x 16` objects cut from the
/// object's grid on the world grid at the placement's translation, summed
/// down the hierarchy from the roots and rounded to whole voxels. A tile
/// holding no live voxel is not cut. An object placed by no node is tiled
/// once at the origin under a node named for it. An object placed by several
/// nodes is tiled per placement. The ext's layer entry for each node stamps
/// its tiles at their world positions. The original nodes and objects are
/// released. Palettes and value pools stay, and a tile's voxel samples the
/// materials its source voxel did. Every layer entry is the synthesized
/// entry a node retained later also gets.
///
/// Lossy where Goxel cannot represent the source. Grouping collapses because
/// layers do not nest. Rotation and scale drop because only translation
/// survives the flattening.
pub fn to_goxl_vox_main(mut main: VoxMain<()>) -> Result<GoxlVoxMain> {
    let old_node_ids: Vec<_> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .collect();
    let old_object_ids: Vec<_> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .collect();

    let mut placements = Vec::new();
    let mut placed = HashSet::new();
    for &root_id in main.root_hierarchy_node_ids() {
        push_placements(
            &main,
            root_id,
            TyVector3I32::new(0, 0, 0),
            &mut placed,
            &mut placements,
        );
    }
    for (object_id, object) in main.iter_objects() {
        if !placed.contains(&object_id) {
            placements.push(Placement {
                name: object.name().to_owned(),
                object_id,
                world: TyVector3I32::new(0, 0, 0),
            });
        }
    }

    // One layer per placement, each stamping the tiles cut for it.
    let mut layers = Vec::with_capacity(placements.len());
    for placement in placements {
        let tiles = {
            let object = main
                .object(placement.object_id)
                .expect("a placed object is one of the main's");
            tiles(object, placement.world)?
        };

        let mut stamps = Vec::with_capacity(tiles.len());
        for (origin, tile) in tiles {
            stamps.push(Stamp {
                tile_id: main.retain_object(tile)?,
                origin,
            });
        }
        layers.push(Layer {
            name: placement.name,
            stamps,
        });
    }

    let node_ids = main.retain_hierarchy_nodes(
        layers
            .iter()
            .map(|layer| VoxHierarchyNode {
                name: layer.name.clone(),
                child_object_ids: layer.stamps.iter().map(|stamp| stamp.tile_id).collect(),
                ..Default::default()
            })
            .collect(),
    )?;
    main.set_root_hierarchy_node_ids(node_ids.clone())?;

    // The old nodes are unlinked first so each releases in any order.
    for &node_id in &old_node_ids {
        let mut node = main.hierarchy_node(node_id).expect("a listed node").clone();
        node.child_node_ids.clear();
        main.set_hierarchy_node(node_id, node)?;
    }
    for node_id in old_node_ids {
        main.release_hierarchy_node(node_id)?;
    }
    for object_id in old_object_ids {
        main.release_object(object_id)?;
    }

    // The ext of a default file, then one entry per layer node.
    let mut ext = goxl_ext_from_file(&GoxlFile::default());
    for (node_id, layer) in node_ids.into_iter().zip(layers) {
        let placements = layer
            .stamps
            .iter()
            .map(|stamp| GoxlExtPlacement {
                object_id: stamp.tile_id,
                position: stamp.origin,
            })
            .collect();
        ext.layers.insert(
            node_id,
            synthesized_layer(next_layer_id(&ext.layers), placements),
        );
    }

    Ok(main.put_ext(ext))
}

/// A layer to synthesize: the placing node's name and the tiles it stamps.
struct Layer {
    name: String,
    stamps: Vec<Stamp>,
}

/// One tile object stamped at the world position of its lower corner.
struct Stamp {
    tile_id: U32Id<BVoxObject>,
    origin: [i32; 3],
}

/// One live voxel cut into a tile: its position in the tile and the
/// materials it samples, one per layer.
struct Cell {
    local: TyVector3U32,
    samples: Vec<U32Id<BVoxMaterial>>,
}

/// One object placement to tile: the placing node's name and the world
/// translation the placement lands at.
struct Placement {
    name: String,
    object_id: U32Id<BVoxObject>,
    world: TyVector3I32,
}

/// Walks one hierarchy node, summing its translation into the world position,
/// recording a placement for each object it places, then recursing into its
/// child nodes.
fn push_placements(
    main: &VoxMain<()>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent: TyVector3I32,
    placed: &mut HashSet<U32Id<BVoxObject>>,
    placements: &mut Vec<Placement>,
) {
    let node = main
        .hierarchy_node(node_id)
        .expect("a root or child is a listed node");
    let world = parent + node.transform.position.round().as_ivec3();

    for &object_id in &node.child_object_ids {
        placed.insert(object_id);
        placements.push(Placement {
            name: node.name.clone(),
            object_id,
            world,
        });
    }
    for &child_id in &node.child_node_ids {
        push_placements(main, child_id, world, placed, placements);
    }
}

/// Cuts `object` into `16 x 16 x 16` tiles on the world grid with the object
/// at `world`, each with the world position of its lower corner, in position
/// order. A tile keeps the object's layers, and each of its voxels samples
/// what the source voxel did. A tile holding no live voxel is not cut.
fn tiles(object: &VoxObject, world: TyVector3I32) -> Result<Vec<([i32; 3], VoxObject)>> {
    let layers: Vec<_> = object.iter_layers().collect();
    let edge = GoxlBlock::SIZE as i32;

    let mut cells: BTreeMap<[i32; 3], Vec<Cell>> = BTreeMap::new();
    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        let world_position = (world + position.as_ivec3()).to_array();
        let origin = [
            world_position[0].div_euclid(edge) * edge,
            world_position[1].div_euclid(edge) * edge,
            world_position[2].div_euclid(edge) * edge,
        ];
        let local = TyVector3U32::new(
            (world_position[0] - origin[0]) as u32,
            (world_position[1] - origin[1]) as u32,
            (world_position[2] - origin[2]) as u32,
        );
        let samples = layers
            .iter()
            .map(|&(layer_id, _)| {
                object
                    .voxel_material(voxel_id, layer_id)
                    .expect("a live voxel samples every layer")
            })
            .collect();
        cells
            .entry(origin)
            .or_default()
            .push(Cell { local, samples });
    }

    cells
        .into_iter()
        .map(|(origin, cells)| {
            let mut tile = VoxObject::new(
                object.name().to_owned(),
                TyVector3U32::splat(GoxlBlock::SIZE),
            )?;

            // A layer's default material only backs empty cells, so the first
            // cut voxel's sample serves.
            for (index, &(_, palette_id)) in layers.iter().enumerate() {
                tile.retain_layer(palette_id, cells[0].samples[index]);
            }
            for cell in &cells {
                let voxel_id = tile
                    .voxel_id(cell.local)
                    .expect("a cut cell is inside the tile");
                tile.retain_voxel(voxel_id, &cell.samples)?;
            }

            Ok((origin, tile))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{GoxlExtPlacement, from_goxl_file, to_goxl_file, to_goxl_vox_main};
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

    fn placement(object_index: u32, position: [i32; 3]) -> GoxlExtPlacement {
        GoxlExtPlacement {
            object_id: U32Id::from_u32(object_index),
            position,
        }
    }

    /// The linear-light components of a `#RRGGBBAA` hex string.
    fn linear_rgba(hex: &str) -> [f64; 4] {
        lin_srgba_f64_from_srgba_u8(TySrgbaU8::from_hex(hex).expect("a valid hex color")).into()
    }

    /// A bare main built straight from voxcore: a red-green object and a
    /// blue object sharing one `rgba` palette, placed by a hierarchy of a
    /// nested group and two roots. This is the cross-format synthesis input.
    fn source_main() -> VoxMain<()> {
        let mut main = VoxMain::default();

        // One baseColor palette: a transparent placeholder, then red,
        // green, blue.
        let value_pool_id = main.retain_value_pool(
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
        let palette_id = main.retain_palette(palette).unwrap();
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
        main.retain_object(wide).unwrap();

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.retain_layer(palette_id, material_id(0));
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(3)])
            .expect("one sample for the one layer");
        main.retain_object(unit).unwrap();

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
        main.retain_hierarchy_nodes(vec![
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
        main.set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        main.validate().expect("a well-formed source main");
        main
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

    /// A default main has no objects, so the writer synthesizes an empty
    /// file.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let main = to_goxl_vox_main(VoxMain::default()).unwrap();
        let file = to_goxl_file(&main).unwrap();
        assert!(file.blocks.is_empty());
        assert!(file.layers.is_empty());
    }

    /// A bare main, such as one cross-loaded from another format, synthesizes
    /// a file: the hierarchy flattens to layers of placed blocks whose world
    /// voxels and colors match the source, one layer per placement named for
    /// its node, and the file reads back into a valid main.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let main = to_goxl_vox_main(source_main()).unwrap();
        let file = to_goxl_file(&main).unwrap();

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
        // object in a valid main.
        assert_eq!(file.blocks.len(), 2);
        let reloaded = from_goxl_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// The synthesized main has the shape a loaded file has: one root node
    /// per placement with no transform, one tile object per stamped block,
    /// and a layer entry stamping each tile at its world position. The
    /// source objects and nodes are gone.
    #[test]
    fn synthesizes_a_layer_entry_per_placement() {
        let main = to_goxl_vox_main(source_main()).unwrap();

        assert_eq!(main.hierarchy_node_count(), 2);

        assert_eq!(main.object_count(), 2);

        assert!(main.object(U32Id::from_u32(0)).is_none());

        assert!(main.hierarchy_node(U32Id::from_u32(0)).is_none());

        let ext = main.ext();

        assert_eq!(ext.layers.len(), 2);

        let wide = &ext.layers[&U32Id::from_u32(3)];

        assert_eq!(wide.id, 1);

        assert_eq!(wide.placements, vec![placement(2, [0, 0, 0])]);

        let unit = &ext.layers[&U32Id::from_u32(4)];

        assert_eq!(unit.id, 2);

        assert_eq!(unit.placements, vec![placement(3, [0, 0, 0])]);

        let node = main.hierarchy_node(U32Id::from_u32(3)).unwrap();

        assert_eq!(node.name, "wide");

        assert_eq!(node.transform, TyTransformF64::default());

        assert_eq!(
            node.child_object_ids,
            vec![U32Id::<BVoxObject>::from_u32(2)]
        );
    }

    /// A node retained on the synthesized state gets the entry the
    /// synthesizer would have built for it, so the ext stays complete.
    #[test]
    fn a_node_retained_after_synthesis_gets_a_synthesized_entry() {
        let mut main = to_goxl_vox_main(source_main()).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "later".to_owned(),
                child_object_ids: vec![U32Id::from_u32(3)],
                transform: TyTransformF64::new(
                    TyVector3F64::new(16.0, 0.0, 0.0),
                    TyQuaternionF64::IDENTITY,
                    TyVector3F64::new(1.0, 1.0, 1.0),
                ),
                ..Default::default()
            })
            .unwrap();

        main.push_root_hierarchy_node_id(node_id).unwrap();

        let later = &main.ext().layers[&node_id];

        assert_eq!(later.id, 3);

        assert_eq!(later.placements, vec![placement(3, [16, 0, 0])]);

        let file = to_goxl_file(&main).unwrap();

        assert_eq!(file.layers[2].name, "later");

        assert!(world_voxels(&file).contains(&(16, 3, 0, (0, 0, 0xFF, 0xFF))));
    }

    /// An object wider than a block is cut into tiles on the world grid. The
    /// file places the same world voxels, and each tile writes as one block.
    #[test]
    fn retiles_an_object_larger_than_a_block() {
        let mut main = VoxMain::default();

        let value_pool_id = main
            .retain_value_pool(VoxValuePool::vec_4_float(vec![linear_rgba("#FF0000FF")]).unwrap());

        let mut palette = VoxPalette::default();

        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();

        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        let palette_id = main.retain_palette(palette).unwrap();

        // A 20-wide object with a voxel at each end, placed at +10x. The
        // ends land at world x 10 and 29, in two tiles.
        let mut wide = VoxObject::new("wide".to_owned(), TyVector3U32::new(20, 1, 1)).unwrap();

        wide.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));

        for x in [0, 19] {
            let voxel_id = wide.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();

            wide.retain_voxel(voxel_id, &[U32Id::from_u32(0)]).unwrap();
        }

        let object_id = main.retain_object(wide).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "wide".to_owned(),
                child_object_ids: vec![object_id],
                transform: TyTransformF64::new(
                    TyVector3F64::new(10.0, 0.0, 0.0),
                    TyQuaternionF64::IDENTITY,
                    TyVector3F64::new(1.0, 1.0, 1.0),
                ),
                ..Default::default()
            })
            .unwrap();

        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();

        let main = to_goxl_vox_main(main).unwrap();

        assert_eq!(main.object_count(), 2);

        let layer = &main.ext().layers[&U32Id::from_u32(1)];

        assert_eq!(
            layer.placements,
            vec![placement(1, [0, 0, 0]), placement(2, [16, 0, 0])]
        );

        for (tile_id, x) in [(1u32, 10u32), (2, 13)] {
            let tile = main.object(U32Id::from_u32(tile_id)).unwrap();

            assert_eq!(tile.bounds(), TyVector3U32::splat(16));

            assert_eq!(tile.live_count(), 1);

            let voxel_id = tile.iter_live().next().unwrap();

            assert_eq!(
                tile.voxel_position(voxel_id),
                Some(TyVector3U32::new(x, 0, 0))
            );
        }

        let file = to_goxl_file(&main).unwrap();

        let red = (0xFF, 0, 0, 0xFF);

        assert_eq!(
            world_voxels(&file),
            BTreeSet::from([(10, 0, 0, red), (29, 0, 0, red)])
        );

        assert_eq!(file.blocks.len(), 2);
    }

    /// An object placed by two nodes is tiled per placement, and an object no
    /// node places is tiled once at the origin under a node named for it.
    #[test]
    fn tiles_per_placement_and_places_an_unplaced_object_at_the_origin() {
        let mut main = source_main();

        let unit_id = U32Id::<BVoxObject>::from_u32(1);

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "again".to_owned(),
                child_object_ids: vec![unit_id],
                transform: TyTransformF64::new(
                    TyVector3F64::new(0.0, 0.0, 20.0),
                    TyQuaternionF64::IDENTITY,
                    TyVector3F64::new(1.0, 1.0, 1.0),
                ),
                ..Default::default()
            })
            .unwrap();

        main.push_root_hierarchy_node_id(node_id).unwrap();

        let mut loose = VoxObject::new("loose".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        loose.retain_layer(
            U32Id::<voxcore::BVoxPalette>::from_u32(0),
            U32Id::<BVoxMaterial>::from_u32(0),
        );

        let voxel_id = loose.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

        loose.retain_voxel(voxel_id, &[U32Id::from_u32(2)]).unwrap();

        main.retain_object(loose).unwrap();

        let main = to_goxl_vox_main(main).unwrap();

        let names: Vec<&str> = main
            .iter_hierarchy_nodes()
            .map(|(_, node)| node.name.as_str())
            .collect();

        assert_eq!(names, ["wide", "unit", "again", "loose"]);

        let again = &main.ext().layers[&U32Id::from_u32(6)];

        assert_eq!(again.placements, vec![placement(5, [0, 0, 16])]);

        let loose = &main.ext().layers[&U32Id::from_u32(7)];

        assert_eq!(loose.placements, vec![placement(6, [0, 0, 0])]);

        let file = to_goxl_file(&main).unwrap();

        let green = (0, 0xFF, 0, 0xFF);

        let blue = (0, 0, 0xFF, 0xFF);

        assert!(world_voxels(&file).contains(&(0, 0, 20, blue)));

        assert!(world_voxels(&file).contains(&(0, 0, 0, green)));
    }
}
