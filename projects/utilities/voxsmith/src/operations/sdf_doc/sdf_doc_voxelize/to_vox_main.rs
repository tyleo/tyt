use crate::{
    Error, Result,
    operations::sdf_doc::{
        SdfCell, SdfGrid, SdfPlace, SdfSampling, SdfVoxMainOptions, is_exposed, retain_sdf_palette,
        see_through_materials,
    },
    utilities::{FillMode, FlattenMode, VoxelFrame},
};
use branded_id::{IdSlice, IdVec, IteratorExt, U32Id};
use sdfcore::BSdfMaterial;
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use voxcore::{BVoxHierarchyNode, BVoxObject, BVoxVoxel, VoxHierarchyNode, VoxMain, VoxObject};

/// How far a node's offset to its object can sit from a whole number of voxels
/// and still count as whole. The division by the voxel size rounds.
const WHOLE_TOLERANCE: f64 = 1e-9;

/// The voxj document of the grids in `sampling` under `options` by model
/// evaluation. Errors on flattening grids sampled under the local frame.
pub fn to_vox_main(sampling: &SdfSampling, options: &SdfVoxMainOptions) -> Result<VoxMain> {
    if options.flatten != FlattenMode::None && sampling.frame == VoxelFrame::Local {
        return Err(Error::invalid(
            "flatten must read grids sampled under the world frame, not the local frame",
        ));
    }

    let see_through = see_through_materials(&sampling.materials);

    let kept: Vec<Vec<SdfCell>> = sampling
        .grids
        .iter()
        .map(|grid| match options.fill_mode {
            FillMode::Solid => grid.cells.clone(),
            FillMode::Surface => surface_cells(grid, &see_through),
        })
        .collect();

    let (objects, nodes) = match options.flatten {
        FlattenMode::None => placed(sampling, &kept),
        FlattenMode::Nodes | FlattenMode::Objects => flattened(sampling, &kept, options.flatten)?,
    };

    let mut first_met = Vec::new();
    let mut met = HashSet::new();

    for object in &objects {
        for material_id in object.materials.iter().flatten() {
            if met.insert(*material_id) {
                first_met.push(*material_id);
            }
        }
    }

    let mut main = VoxMain::default();
    let (palette_id, palette_materials) =
        retain_sdf_palette(&mut main, &sampling.materials, &first_met)?;

    let mut object_ids = Vec::with_capacity(objects.len());

    for object in &objects {
        let mut vox_object = VoxObject::new(object.name.clone(), object.size)
            .expect("a part's grid fits voxcore's cell limit");
        vox_object.set_origin(object.min + object.base);
        vox_object
            .retain_layer(palette_id)
            .expect("a fresh object has no live voxel");

        for (voxel_id, material_id) in object.materials.iter().enumerate_ids() {
            if let Some(material_id) = material_id {
                vox_object
                    .retain_voxel(voxel_id, &[palette_materials[material_id]])
                    .expect("a raster index is a voxel of the object sampling its one layer");
            }
        }

        object_ids.push(main.retain_object(vox_object)?);
    }

    let mut node_ids: Vec<U32Id<BVoxHierarchyNode>> = Vec::with_capacity(nodes.len());
    let mut children = Vec::with_capacity(nodes.len());

    for node in &nodes {
        let node_id = main.retain_hierarchy_node(VoxHierarchyNode {
            name: node.name.clone(),
            transform: node.transform,
            ..VoxHierarchyNode::default()
        })?;
        node_ids.push(node_id);

        let mut child_object_ids = Vec::new();
        let mut voxels: Vec<(TyVector3F64, Vec<U32Id<BVoxObject>>)> = Vec::new();

        for &object_index in &node.object_indices {
            let residual = snap(node.lattice_offset - objects[object_index].base.as_dvec3());

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
            nodes
                .iter()
                .enumerate()
                .filter(|(_, child)| child.parent == Some(index))
                .map(|(child_index, _)| node_ids[child_index]),
        );

        main.set_hierarchy_node_children(node_ids[index], child_node_ids, child_object_ids)?;

        if nodes[index].parent.is_none() {
            main.push_root_hierarchy_node_id(node_ids[index])?;
        }
    }

    Ok(main)
}

/// A live cell's lattice index beside its material.
type LiveCell = (TyVector3I32, U32Id<BSdfMaterial>);

/// One object the document holds.
struct ObjectCells {
    /// The part's name.
    name: String,

    /// The lattice index of the least cell of the box around the live cells.
    min: TyVector3I32,

    /// The cells along each axis of the box around the live cells.
    size: TyVector3U32,

    /// The material of each cell of the box in a raster with x outermost, or
    /// `None` for an empty cell.
    materials: IdVec<BVoxVoxel, Option<U32Id<BSdfMaterial>>>,

    /// The whole voxels the object's origin adds to `min`. The object's first
    /// place fixes them.
    base: TyVector3I32,
}

/// One node the document holds for a place.
struct PlaceNode {
    /// The part's name.
    name: String,

    /// The index of the node of the place this one sits under, or `None` for a
    /// root node.
    parent: Option<usize>,

    transform: TyTransformF64,

    /// The objects at the place.
    object_indices: Vec<usize>,

    /// The offset in voxels from a lattice index to the node's frame: `f` in
    /// model evaluation.
    lattice_offset: TyVector3F64,
}

/// The objects and nodes of each place under the frame `sampling` was sampled
/// in.
fn placed(sampling: &SdfSampling, kept: &[Vec<SdfCell>]) -> (Vec<ObjectCells>, Vec<PlaceNode>) {
    let mut objects = Vec::new();
    let mut grid_objects: HashMap<usize, Option<usize>> = HashMap::new();
    let mut nodes = Vec::with_capacity(sampling.places.len());

    for place in &sampling.places {
        let shift = match sampling.frame {
            VoxelFrame::World => place.offset,
            VoxelFrame::Local => TyVector3F64::ZERO,
        };
        let lattice_offset = -(place.pivot - place.offset + shift) / sampling.voxel_size;
        let name = place.path.last().expect("a place has a part").clone();

        let object_indices = place
            .grid_indices
            .iter()
            .filter_map(|&grid_index| {
                *grid_objects.entry(grid_index).or_insert_with(|| {
                    let grid = &sampling.grids[grid_index];
                    let cells = grid_cells(grid, &kept[grid_index]);
                    let object = object_cells(name.clone(), &cells, lattice_offset)?;
                    objects.push(object);
                    Some(objects.len() - 1)
                })
            })
            .collect();

        nodes.push(PlaceNode {
            name,
            parent: place.parent,
            transform: node_transform(sampling, place),
            object_indices,
            lattice_offset,
        });
    }

    (objects, nodes)
}

/// One node for each root part on the root part's grid. Under
/// [`FlattenMode::Nodes`] the node holds the object of each place at or below
/// the root part. Under [`FlattenMode::Objects`] the node holds one object of
/// those places' live cells, and a part placed later wins a cell.
fn flattened(
    sampling: &SdfSampling,
    kept: &[Vec<SdfCell>],
    flatten: FlattenMode,
) -> Result<(Vec<ObjectCells>, Vec<PlaceNode>)> {
    let mut roots: Vec<usize> = Vec::with_capacity(sampling.places.len());

    for (index, place) in sampling.places.iter().enumerate() {
        if place.parent.is_some() && place.rotation != TyQuaternionF64::IDENTITY {
            return Err(Error::invalid(format!(
                "{}: flattening drops the part's node, which carries its rotation",
                place.path.join("/"),
            )));
        }

        roots.push(place.parent.map_or(index, |parent| roots[parent]));
    }

    let mut objects = Vec::new();
    let mut nodes = Vec::new();

    for (root, place) in sampling.places.iter().enumerate() {
        if place.parent.is_some() {
            continue;
        }

        let name = place.path[0].clone();
        let lattice_offset = -place.pivot / sampling.voxel_size;

        let below: Vec<(&String, Vec<LiveCell>)> = sampling
            .places
            .iter()
            .enumerate()
            .filter(|(index, _)| roots[*index] == root)
            .flat_map(|(_, place)| {
                let part = place.path.last().expect("a place has a part");

                place.grid_indices.iter().map(move |&grid_index| {
                    (
                        part,
                        grid_cells(&sampling.grids[grid_index], &kept[grid_index]),
                    )
                })
            })
            .collect();

        let first_object = objects.len();

        if flatten == FlattenMode::Objects {
            let cells: Vec<LiveCell> = below.into_iter().flat_map(|(_, cells)| cells).collect();

            if let Some((min, max)) = cell_box(&cells) {
                let volume = VoxObject::volume_of((max - min + 1).as_uvec3());

                if volume > VoxObject::MAX_GRID_CELLS {
                    return Err(Error::invalid(format!(
                        "{name}: flattened object must hold at most {} cells, not {volume}",
                        VoxObject::MAX_GRID_CELLS,
                    )));
                }
            }

            objects.extend(object_cells(name.clone(), &cells, lattice_offset));
        } else {
            objects.extend(
                below
                    .into_iter()
                    .filter_map(|(part, cells)| object_cells(part.clone(), &cells, lattice_offset)),
            );
        }

        nodes.push(PlaceNode {
            name,
            parent: None,
            transform: node_transform(sampling, place),
            object_indices: (first_object..objects.len()).collect(),
            lattice_offset,
        });
    }

    Ok((objects, nodes))
}

/// The live cells of `grid` holding `cells`, by lattice index, in a raster
/// with x outermost.
fn grid_cells(grid: &SdfGrid, cells: &[SdfCell]) -> Vec<LiveCell> {
    let size = grid.size.as_ivec3();

    cells
        .iter()
        .enumerate()
        .filter_map(|(index, cell)| {
            let index = index as i32;
            let offset = TyVector3I32::new(
                index / (size.y * size.z),
                index / size.z % size.y,
                index % size.z,
            );

            cell.material
                .map(|material_id| (grid.min + offset, material_id))
        })
        .collect()
}

/// The least and greatest lattice indices of `cells`, or `None` for no cells.
fn cell_box(cells: &[LiveCell]) -> Option<(TyVector3I32, TyVector3I32)> {
    cells.iter().fold(None, |bounds, (cell, _)| {
        Some(bounds.map_or(
            (*cell, *cell),
            |(min, max): (TyVector3I32, TyVector3I32)| (min.min(*cell), max.max(*cell)),
        ))
    })
}

/// The object named `name` holding `cells`, or `None` for no cells. A later
/// cell at one lattice index wins it. The object's first place sits
/// `lattice_offset` voxels from the lattice.
fn object_cells(
    name: String,
    cells: &[LiveCell],
    lattice_offset: TyVector3F64,
) -> Option<ObjectCells> {
    let (min, max) = cell_box(cells)?;
    let size = (max - min + 1).as_uvec3();
    let mut materials = IdVec::from_vec(vec![None; VoxObject::volume_of(size) as usize]);

    for (cell, material_id) in cells {
        let position = (*cell - min).as_uvec3();
        let voxel_id =
            VoxObject::raster_id(size, position).expect("a live cell lies inside its box");
        materials[voxel_id.to_usize_id()] = Some(*material_id);
    }

    let base = TyVector3F64::from_array(lattice_offset.to_array().map(|component| {
        let rounded = component.round();

        if (component - rounded).abs() <= WHOLE_TOLERANCE {
            rounded
        } else {
            component.floor()
        }
    }))
    .as_ivec3();

    Some(ObjectCells {
        name,
        min,
        size,
        materials,
        base,
    })
}

/// The transform of `place`'s node.
fn node_transform(sampling: &SdfSampling, place: &SdfPlace) -> TyTransformF64 {
    match place.parent {
        Some(parent) => TyTransformF64 {
            position: (place.pivot - sampling.places[parent].pivot) / sampling.voxel_size,
            rotation: place.rotation,
            ..TyTransformF64::IDENTITY
        },

        None => TyTransformF64 {
            position: place.pivot,
            rotation: place.rotation,
            scale: TyVector3F64::splat(sampling.voxel_size),
        },
    }
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

/// The cells of `grid` with each hidden live cell emptied.
fn surface_cells(grid: &SdfGrid, see_through: &IdSlice<BSdfMaterial, bool>) -> Vec<SdfCell> {
    grid.cells
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            if cell.material.is_none() {
                return *cell;
            }

            if is_exposed(grid, grid.lattice_index(index), see_through) {
                *cell
            } else {
                SdfCell {
                    material: None,
                    ..*cell
                }
            }
        })
        .collect()
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        dependencies::DependenciesImpl,
        operations::sdf_doc::{
            SdfSampleOptions, SdfVoxMainOptions, parts_main, sample, shared_leaf_main,
            single_part_main, to_vox_main,
        },
        utilities::{FillMode, FlattenMode, GridResolution, VoxelFrame},
    };
    use branded_id::{IdVec, U32Id};
    use sdfcore::{
        SdfAxisRotation, SdfMain, SdfMaterial, SdfNode, SdfObject, SdfProperty, SdfPropertyValue,
        SdfShape3d, SdfState, SdfStep, SdfStepMaterial,
    };
    use ty_math::{TyAxis3, TyVector3F64};
    use voxcore::VoxMain;

    /// The document of `main` sampled at one meter per voxel under `frame`.
    fn write(
        main: &SdfMain,
        frame: VoxelFrame,
        fill_mode: FillMode,
        flatten: FlattenMode,
    ) -> VoxMain {
        let sampling = sample(
            &DependenciesImpl,
            main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame,
                record_writes: false,
            },
        )
        .unwrap();

        let document = to_vox_main(&sampling, &SdfVoxMainOptions { fill_mode, flatten }).unwrap();
        assert_eq!(document.validate(), Ok(()));
        document
    }

    /// Each node in id order with its position, scale, child nodes, and child
    /// objects.
    fn nodes(document: &VoxMain) -> Vec<String> {
        let node_name = |node_id| document.hierarchy_node(node_id).unwrap().name.clone();
        let object_name = |object_id| document.object(object_id).unwrap().name().to_owned();

        document
            .iter_hierarchy_nodes()
            .map(|(_, node)| {
                let position = node.transform.position;
                let children: Vec<String> = node
                    .child_node_ids
                    .iter()
                    .map(|id| node_name(*id))
                    .collect();
                let objects: Vec<String> = node
                    .child_object_ids
                    .iter()
                    .map(|id| object_name(*id))
                    .collect();

                format!(
                    "{} at [{}, {}, {}] by {}: nodes {children:?}, objects {objects:?}",
                    node.name, position.x, position.y, position.z, node.transform.scale.x
                )
            })
            .collect()
    }

    /// Each object in id order with its origin, bounds, and live voxels.
    fn objects(document: &VoxMain) -> Vec<String> {
        document
            .iter_objects()
            .map(|(_, object)| {
                let origin = object.origin();
                let bounds = object.bounds();

                format!(
                    "{} from [{}, {}, {}] across {}x{}x{}: {} voxels",
                    object.name(),
                    origin.x,
                    origin.y,
                    origin.z,
                    bounds.x,
                    bounds.y,
                    bounds.z,
                    object.live_count()
                )
            })
            .collect()
    }

    #[test]
    fn each_part_writes_its_object_under_a_node_at_its_pivot() {
        let document = write(
            &parts_main(),
            VoxelFrame::World,
            FillMode::Solid,
            FlattenMode::None,
        );

        assert_eq!(
            nodes(&document),
            [
                r#"model at [0, 0, 0] by 1: nodes ["lid", "pebble"], objects ["model"]"#,
                r#"lid at [2, 2, 2] by 1: nodes [], objects ["lid"]"#,
                r#"pebble at [6, 0, 0] by 1: nodes [], objects ["pebble"]"#,
            ]
        );
        assert_eq!(
            objects(&document),
            [
                "model from [0, 0, 0] across 4x2x4: 32 voxels",
                "lid from [-1, 0, -1] across 2x1x2: 4 voxels",
                "pebble from [0, 0, 0] across 1x1x1: 1 voxels",
            ]
        );
        assert_eq!(document.root_hierarchy_node_ids(), [U32Id::from_u32(0)]);
    }

    #[test]
    fn a_pivot_off_the_lattice_places_the_object_through_a_voxels_node() {
        let world = write(
            &shared_leaf_main(),
            VoxelFrame::World,
            FillMode::Solid,
            FlattenMode::None,
        );

        assert_eq!(
            nodes(&world),
            [
                r#"model at [0.5, 0, 0] by 1: nodes ["a", "b"], objects []"#,
                r#"a at [2, 0, 0] by 1: nodes ["leaf"], objects []"#,
                r#"leaf at [0, 0, 0] by 1: nodes ["voxels"], objects []"#,
                r#"voxels at [0.5, 0, 0] by 1: nodes [], objects ["leaf"]"#,
                r#"b at [0, 0.25, 0] by 1: nodes ["leaf"], objects []"#,
                r#"leaf at [0, 0, 0] by 1: nodes ["voxels"], objects []"#,
                r#"voxels at [0.5, 0.75, 0] by 1: nodes [], objects ["leaf"]"#,
            ]
        );
        assert_eq!(
            objects(&world),
            [
                "leaf from [-1, 0, 0] across 1x1x1: 1 voxels",
                "leaf from [-1, -1, 0] across 1x1x1: 1 voxels",
            ]
        );
    }

    #[test]
    fn every_place_shares_its_part_object_under_the_local_frame() {
        let local = write(
            &shared_leaf_main(),
            VoxelFrame::Local,
            FillMode::Solid,
            FlattenMode::None,
        );

        assert_eq!(
            objects(&local),
            ["leaf from [-1, 0, 0] across 1x1x1: 1 voxels"]
        );
        assert_eq!(
            nodes(&local)[3],
            r#"voxels at [0.5, 0, 0] by 1: nodes [], objects ["leaf"]"#
        );
        assert_eq!(
            nodes(&local)[6],
            r#"voxels at [0.5, 0, 0] by 1: nodes [], objects ["leaf"]"#
        );
    }

    /// `parts_main` with the lid rotated 90 degrees about y, then 90 about x.
    fn rotated_lid_main() -> SdfMain {
        let mut state = parts_main().into_state();
        state.nodes[U32Id::from_u32(0).to_usize_id()].rotation = vec![
            SdfAxisRotation {
                axis: TyAxis3::Y,
                degrees: 90.0,
            },
            SdfAxisRotation {
                axis: TyAxis3::X,
                degrees: 90.0,
            },
        ];

        SdfMain::new(state).unwrap()
    }

    #[test]
    fn a_part_node_takes_the_part_rotation_in_order_on_the_parent_axes() {
        let document = write(
            &rotated_lid_main(),
            VoxelFrame::World,
            FillMode::Solid,
            FlattenMode::None,
        );

        let (_, lid) = document
            .iter_hierarchy_nodes()
            .find(|(_, node)| node.name == "lid")
            .unwrap();
        let rotation = lid.transform.rotation;

        // The rotation about y carries +x onto -z, and the rotation about x then
        // carries -z onto +y.
        assert!((rotation * TyVector3F64::X).abs_diff_eq(TyVector3F64::Y, 1e-12));
        assert_eq!(lid.transform.position, TyVector3F64::splat(2.0));
    }

    #[test]
    fn flattening_a_rotated_part_errors() {
        let sampling = sample(
            &DependenciesImpl,
            &rotated_lid_main(),
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame: VoxelFrame::World,
                record_writes: false,
            },
        )
        .unwrap();
        let options = SdfVoxMainOptions {
            fill_mode: FillMode::Solid,
            flatten: FlattenMode::Nodes,
        };

        assert_eq!(
            to_vox_main(&sampling, &options).unwrap_err().to_string(),
            "model/lid: flattening drops the part's node, which carries its rotation"
        );
    }

    #[test]
    fn flattening_nodes_keeps_each_part_object_on_the_root_part_grid() {
        let document = write(
            &parts_main(),
            VoxelFrame::World,
            FillMode::Solid,
            FlattenMode::Nodes,
        );

        assert_eq!(
            nodes(&document),
            [r#"model at [0, 0, 0] by 1: nodes [], objects ["model", "lid", "pebble"]"#]
        );
        assert_eq!(
            objects(&document),
            [
                "model from [0, 0, 0] across 4x2x4: 32 voxels",
                "lid from [1, 2, 1] across 2x1x2: 4 voxels",
                "pebble from [6, 0, 0] across 1x1x1: 1 voxels",
            ]
        );
    }

    #[test]
    fn flattening_objects_writes_each_root_part_as_one_object() {
        let document = write(
            &parts_main(),
            VoxelFrame::World,
            FillMode::Solid,
            FlattenMode::Objects,
        );

        assert_eq!(
            nodes(&document),
            [r#"model at [0, 0, 0] by 1: nodes [], objects ["model"]"#]
        );
        assert_eq!(
            objects(&document),
            ["model from [0, 0, 0] across 7x3x4: 37 voxels"]
        );
    }

    #[test]
    fn flattening_needs_the_world_frame() {
        let sampling = sample(
            &DependenciesImpl,
            &parts_main(),
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame: VoxelFrame::Local,
                record_writes: false,
            },
        )
        .unwrap();
        let options = SdfVoxMainOptions {
            fill_mode: FillMode::Solid,
            flatten: FlattenMode::Objects,
        };

        assert_eq!(
            to_vox_main(&sampling, &options).unwrap_err().to_string(),
            "flatten must read grids sampled under the world frame, not the local frame"
        );
    }

    #[test]
    fn a_flattened_object_past_the_cell_limit_errors() {
        let set = |name: &str, point: f64| SdfStep::Set {
            name: name.to_string(),
            points: vec![TyVector3F64::splat(point)],
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
        };
        let node = |name: &str, object: u32, nodes: Vec<u32>| SdfNode {
            name: name.to_string(),
            pivot: None,
            offset: None,
            rotation: Vec::new(),
            child_object_ids: vec![U32Id::from_u32(object)],
            child_node_ids: nodes.into_iter().map(U32Id::from_u32).collect(),
        };

        let main = SdfMain::new(SdfState {
            materials: IdVec::from_vec(vec![SdfMaterial::Material {
                properties: Vec::new(),
            }]),
            steps: IdVec::from_vec(vec![set("near", 0.5), set("far", 600.5)]),
            objects: IdVec::from_vec(vec![
                SdfObject {
                    name: "model".to_string(),
                    step_ids: vec![U32Id::from_u32(0)],
                },
                SdfObject {
                    name: "far".to_string(),
                    step_ids: vec![U32Id::from_u32(1)],
                },
            ]),
            nodes: IdVec::from_vec(vec![node("far", 1, Vec::new()), node("model", 0, vec![0])]),
            root_node_ids: vec![U32Id::from_u32(1)],
            ..SdfState::default()
        })
        .unwrap();

        let sampling = sample(
            &DependenciesImpl,
            &main,
            &SdfSampleOptions {
                resolution: GridResolution::VoxelSize(1.0),
                frame: VoxelFrame::World,
                record_writes: false,
            },
        )
        .unwrap();
        let options = SdfVoxMainOptions {
            fill_mode: FillMode::Solid,
            flatten: FlattenMode::Objects,
        };

        assert_eq!(
            to_vox_main(&sampling, &options).unwrap_err().to_string(),
            "model: flattened object must hold at most 134217728 cells, not 217081801"
        );
    }

    #[test]
    fn the_surface_fill_keeps_the_cells_with_an_empty_face_neighbor() {
        let main = single_part_main(
            vec![SdfShape3d::Box {
                min: TyVector3F64::ZERO,
                max: TyVector3F64::splat(3.0),
                round: None,
            }],
            vec![SdfStep::Add {
                name: "block".to_string(),
                shape_id: U32Id::from_u32(0),
                material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            }],
        );

        for (fill_mode, voxels) in [(FillMode::Solid, 27), (FillMode::Surface, 26)] {
            let document = write(&main, VoxelFrame::World, fill_mode, FlattenMode::None);
            let (_, object) = document.iter_objects().next().unwrap();
            assert_eq!(object.live_count(), voxels);
        }
    }

    #[test]
    fn the_surface_fill_keeps_an_opaque_cell_seen_through_water() {
        // A rock at the center of a 5x5x5 water block.
        let mut state = single_part_main(
            vec![
                SdfShape3d::Box {
                    min: TyVector3F64::ZERO,
                    max: TyVector3F64::splat(5.0),
                    round: None,
                },
                SdfShape3d::Box {
                    min: TyVector3F64::splat(2.0),
                    max: TyVector3F64::splat(3.0),
                    round: None,
                },
            ],
            vec![
                SdfStep::Add {
                    name: "water".to_string(),
                    shape_id: U32Id::from_u32(0),
                    material: SdfStepMaterial::Material(U32Id::from_u32(1)),
                },
                SdfStep::Add {
                    name: "rock".to_string(),
                    shape_id: U32Id::from_u32(1),
                    material: SdfStepMaterial::Material(U32Id::from_u32(0)),
                },
            ],
        )
        .state()
        .clone();
        state.materials[U32Id::from_u32(1).to_usize_id()] = SdfMaterial::Material {
            properties: vec![SdfProperty {
                name: "transmission".to_string(),
                value: SdfPropertyValue::Number(1.0),
            }],
        };
        let main = SdfMain::new(state).unwrap();

        // The 98 shell cells and the rock stay, and the water around the rock
        // goes.
        for (fill_mode, voxels) in [(FillMode::Solid, 125), (FillMode::Surface, 99)] {
            let document = write(&main, VoxelFrame::World, fill_mode, FlattenMode::None);
            let (_, object) = document.iter_objects().next().unwrap();
            assert_eq!(object.live_count(), voxels);
        }
    }

    #[test]
    fn the_palette_merges_identical_materials_and_binds_the_custom_properties_last() {
        let document = write(
            &parts_main(),
            VoxelFrame::World,
            FillMode::Solid,
            FlattenMode::None,
        );
        let (palette_id, palette) = document.iter_palettes().next().unwrap();

        let names: Vec<&str> = palette
            .iter_properties()
            .map(|(_, property)| property.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "baseColor",
                "metallic",
                "roughness",
                "emissiveColor",
                "emissiveStrength",
                "occlusionStrength",
                "ior",
                "transmission",
                "tag",
            ]
        );

        let tag_id = palette.property_id_by_name("tag").unwrap();
        let tags: Vec<String> = palette
            .iter_materials()
            .map(|material_id| {
                let (value_pool, value_id) = document
                    .material_value(palette_id, material_id, tag_id)
                    .unwrap();
                value_pool
                    .string_values()
                    .unwrap()
                    .get(value_id)
                    .unwrap()
                    .clone()
            })
            .collect();
        assert_eq!(tags, ["", "lid"]);
    }
}
