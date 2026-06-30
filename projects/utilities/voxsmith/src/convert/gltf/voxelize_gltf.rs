use crate::{Error, FillMode, Result, gltf_triangles, voxelize_triangles};
use branded_id::U32Id;
use ty_math::{TyTransformF64, TyVector3, TyVector3U32};
use voxcore::{VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValue};

/// Voxelizes a glTF or GLB mesh into a [`VoxMain`] of one object placed by one
/// root node. Errors when the mesh has no triangle geometry, the grid exceeds
/// voxcore's dense-grid limit, or the assembled state fails validation.
///
/// # Arguments
/// * `bytes` - the glTF or GLB mesh; the container is auto-detected.
/// * `counts` - grid resolution in voxels per axis, resolved by the caller from
///   the mesh extent (see [`gltf_mesh_extent`](crate::gltf_mesh_extent)) so the
///   bounding box fits the grid tightly.
/// * `fill_mode` - a solid body (flood-filled) or a hollow surface shell.
/// * `fill_color` - the straight-RGBA color every filled voxel takes, written as
///   the single `rgba` cell of the document's one palette.
/// * `node_scale` - the placing node's uniform scale: the meters-per-voxel size
///   for a scale-derived grid, or `1.0` for a side-length-derived one.
pub fn voxelize_gltf(
    bytes: &[u8],
    counts: TyVector3U32,
    fill_mode: FillMode,
    fill_color: [u8; 4],
    node_scale: f64,
) -> Result<VoxMain> {
    let triangles = gltf_triangles(bytes)?;
    if triangles.is_empty() {
        return Err(Error::invalid("glTF has no triangle geometry to voxelize"));
    }
    // Cap the grid before rasterizing, so an oversized resolution errors rather
    // than overflowing or exhausting memory allocating the occupancy grid.
    let volume = counts.x as u64 * counts.y as u64 * counts.z as u64;
    if volume > VoxObject::MAX_GRID_CELLS {
        return Err(grid_too_large(counts));
    }
    let filled = voxelize_triangles(&triangles, counts, fill_mode == FillMode::Solid);

    let mut state = VoxMain::default();

    // One palette holding the fill color as its single `rgba` cell.
    let mut palette = VoxPalette::default();
    palette.add_attribute("rgba".to_owned());
    let cell = palette
        .add_cell(vec![VoxValue::Text(hex(fill_color))])
        .expect("one value for the one attribute");
    let palette_id = state.add_palette(palette);

    // One object sized to the grid, every filled cell sampling that one cell.
    let mut object =
        VoxObject::new("voxelized".to_owned(), counts).ok_or_else(|| grid_too_large(counts))?;
    object.add_palette_ref(palette_id, cell);
    for (index, &fill) in filled.iter().enumerate() {
        if fill {
            let voxel = U32Id::from_u32(index as u32);
            object
                .retain_voxel(voxel, &[cell])
                .expect("a grid index is a live voxel sampling the one reference");
        }
    }
    let object_id = state.add_object(object);

    // One root node placing the object and carrying the real-world scale.
    let transform = TyTransformF64 {
        scale: TyVector3::new(node_scale, node_scale, node_scale),
        ..Default::default()
    };
    let node = VoxHierarchyNode {
        child_objects: vec![object_id],
        transform,
        ..Default::default()
    };
    let node_id = state.add_hierarchy_node(node);
    state.push_root_hierarchy_node(node_id);

    state.validate()?;
    Ok(state)
}

/// The `#RRGGBBAA` hex string for an RGBA color.
fn hex([r, g, b, a]: [u8; 4]) -> String {
    format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
}

/// The error for a grid past voxcore's dense-grid cell limit.
fn grid_too_large(counts: TyVector3U32) -> Error {
    Error::invalid(format!(
        "voxel grid {}x{}x{} exceeds the dense limit of {} cells",
        counts.x,
        counts.y,
        counts.z,
        VoxObject::MAX_GRID_CELLS
    ))
}

#[cfg(test)]
mod tests {
    use crate::{FillMode, gltf_mesh_extent, voxelize_gltf};
    use ty_math::TyVector3U32;
    use voxcore::VoxValue;

    /// A minimal binary glTF (GLB) of an axis-aligned box spanning `[0, sx]`,
    /// `[0, sy]`, `[0, sz]` in glTF Y-up space, indexed triangles.
    fn box_glb(sx: f32, sy: f32, sz: f32) -> Vec<u8> {
        let verts = [
            [0.0, 0.0, 0.0],
            [sx, 0.0, 0.0],
            [sx, sy, 0.0],
            [0.0, sy, 0.0],
            [0.0, 0.0, sz],
            [sx, 0.0, sz],
            [sx, sy, sz],
            [0.0, sy, sz],
        ];
        let faces = [
            [0u32, 1, 2],
            [0, 2, 3],
            [4, 6, 5],
            [4, 7, 6],
            [0, 4, 5],
            [0, 5, 1],
            [3, 2, 6],
            [3, 6, 7],
            [0, 3, 7],
            [0, 7, 4],
            [1, 5, 6],
            [1, 6, 2],
        ];
        let mut bin = Vec::new();
        for vertex in verts {
            for component in vertex {
                bin.extend_from_slice(&component.to_le_bytes());
            }
        }
        let positions_len = bin.len();
        for face in faces {
            for index in face {
                bin.extend_from_slice(&index.to_le_bytes());
            }
        }
        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],"#,
                r#""nodes":[{{"mesh":0}}],"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0}},"indices":1,"mode":4}}]}}],"#,
                r#""accessors":[{{"bufferView":0,"componentType":5126,"count":8,"type":"VEC3","#,
                r#""min":[0,0,0],"max":[{sx},{sy},{sz}]}},"#,
                r#"{{"bufferView":1,"componentType":5125,"count":36,"type":"SCALAR"}}],"#,
                r#""bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":{positions_len}}},"#,
                r#"{{"buffer":0,"byteOffset":{positions_len},"byteLength":{indices_len}}}],"#,
                r#""buffers":[{{"byteLength":{bin_len}}}]}}"#,
            ),
            sx = sx,
            sy = sy,
            sz = sz,
            positions_len = positions_len,
            indices_len = bin.len() - positions_len,
            bin_len = bin.len(),
        );

        let mut json_bytes = json.into_bytes();
        while json_bytes.len() % 4 != 0 {
            json_bytes.push(b' ');
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        let total = 12 + 8 + json_bytes.len() + 8 + bin.len();

        let mut glb = Vec::new();
        glb.extend_from_slice(&0x4654_6C67u32.to_le_bytes()); // "glTF"
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_bytes);
        glb.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin);
        glb
    }

    #[test]
    fn extent_converts_gltf_y_up_to_voxel_json_z_up() {
        // A box tall on glTF +Y should be tall on Voxel Json +Z.
        let extent = gltf_mesh_extent(&box_glb(1.0, 4.0, 1.0)).unwrap();
        assert!((extent.x - 1.0).abs() < 1e-6, "x extent {}", extent.x);
        assert!((extent.y - 1.0).abs() < 1e-6, "y extent {}", extent.y);
        assert!((extent.z - 4.0).abs() < 1e-6, "z extent {}", extent.z);
    }

    #[test]
    fn voxelizes_a_solid_box_into_one_object_and_palette() {
        let state = voxelize_gltf(
            &box_glb(1.0, 4.0, 1.0),
            TyVector3U32::new(1, 1, 4),
            FillMode::Solid,
            [255, 0, 0, 255],
            2.0,
        )
        .unwrap();
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.object_count(), 1);
        assert_eq!(state.palette_count(), 1);

        let (_, object) = state.iter_objects().next().unwrap();
        assert_eq!(object.bounds(), TyVector3U32::new(1, 1, 4));
        assert_eq!(object.live_count(), 4);

        // The one palette holds the fill color as its single rgba cell.
        let (_, palette) = state.iter_palettes().next().unwrap();
        let (attribute, name) = palette.iter_attributes().next().unwrap();
        assert_eq!(name, "rgba");
        let cell = palette.iter_cells().next().unwrap();
        assert_eq!(
            palette.cell_value(cell, attribute),
            Some(&VoxValue::Text("#FF0000FF".to_owned()))
        );

        // The root node records the meters-per-voxel scale.
        let root = state.root_hierarchy_nodes()[0];
        assert_eq!(state.hierarchy_node(root).unwrap().transform.scale.z, 2.0);
    }

    #[test]
    fn rejects_a_grid_past_the_dense_limit() {
        // 1024^3 = 2^30 cells, well past the 2^27 dense limit; this must error
        // before allocating the occupancy grid rather than exhaust memory.
        let result = voxelize_gltf(
            &box_glb(1.0, 1.0, 1.0),
            TyVector3U32::new(1024, 1024, 1024),
            FillMode::Solid,
            [255, 255, 255, 255],
            1.0,
        );
        assert!(result.is_err());
    }
}
