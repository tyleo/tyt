use crate::{Error, Mesh, MeshMaterial, MeshTriangle, Result};
use gltf::{Material, Node, buffer::Data, import_slice, mesh::Mode};
use std::collections::HashMap;
use ty_math::{TyLinearRgbaColorF64, TyMatrix4x4F64, TyVector3F64};

/// Reads a glTF or GLB byte slice into a [`Mesh`]: every triangle in world
/// space, tagged with its per-primitive material, with glTF's Y-up axes
/// converted to the Voxel Json Z-up convention so a voxelized model stands
/// upright.
///
/// Node and scene transforms are applied, so an authored scale bakes into the
/// points and two exports of one object at different scales voxelize alike.
/// Only `triangles`-mode primitives contribute; points, lines, and triangle
/// strips and fans are skipped. Distinct glTF materials are deduplicated by
/// index, so primitives sharing a material share one material entry, and every
/// primitive with no assigned material shares the glTF default. A `.gltf` that
/// references external buffer files cannot be resolved from bytes alone and
/// errors.
pub fn from_gltf_bytes(bytes: &[u8]) -> Result<Mesh> {
    let (document, buffers, _images) = import_slice(bytes)?;

    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or_else(|| Error::invalid("glTF has no scene"))?;

    let mut mesh = Mesh {
        triangles: Vec::new(),
        materials: Vec::new(),
        name: None,
    };

    let mut slots: HashMap<Option<usize>, u32> = HashMap::new();

    let identity = TyMatrix4x4F64::identity();

    for node in scene.nodes() {
        collect_node(&node, &identity, &buffers, &mut mesh, &mut slots);
    }

    Ok(mesh)
}

/// Appends `node`'s triangles in world space, then recurses into its children.
/// `parent` is the accumulated world transform of `node`'s parent.
fn collect_node(
    node: &Node,
    parent: &TyMatrix4x4F64,
    buffers: &[Data],
    mesh: &mut Mesh,
    slots: &mut HashMap<Option<usize>, u32>,
) {
    let local = TyMatrix4x4F64::from_column_arrays(
        node.transform()
            .matrix()
            .map(|column| column.map(f64::from)),
    );
    let world = *parent * local;

    if let Some(node_mesh) = node.mesh() {
        // Name the object after the first mesh-bearing node, its own name
        // preferred over its mesh's.
        if mesh.name.is_none() {
            mesh.name = node.name().or_else(|| node_mesh.name()).map(str::to_owned);
        }

        for primitive in node_mesh.primitives() {
            if primitive.mode() != Mode::Triangles {
                continue;
            }

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()][..]));

            let Some(positions) = reader.read_positions() else {
                continue;
            };

            let positions: Vec<TyVector3F64> = positions
                .map(|p| world_z_up(&world, [p[0] as f64, p[1] as f64, p[2] as f64]))
                .collect();

            let material = material_slot(&primitive.material(), mesh, slots);

            match reader.read_indices() {
                Some(indices) => {
                    let indices: Vec<u32> = indices.into_u32().collect();
                    for face in indices.chunks_exact(3) {
                        if let (Some(&a), Some(&b), Some(&c)) = (
                            positions.get(face[0] as usize),
                            positions.get(face[1] as usize),
                            positions.get(face[2] as usize),
                        ) {
                            mesh.triangles.push(MeshTriangle {
                                points: [a, b, c],
                                material,
                            });
                        }
                    }
                }

                None => {
                    for face in positions.chunks_exact(3) {
                        mesh.triangles.push(MeshTriangle {
                            points: [face[0], face[1], face[2]],
                            material,
                        });
                    }
                }
            }
        }
    }

    for child in node.children() {
        collect_node(&child, &world, buffers, mesh, slots);
    }
}

/// The material-table slot for `material`, reading its factors and interning it
/// on first sight so primitives sharing a glTF material share one entry.
fn material_slot(
    material: &Material,
    mesh: &mut Mesh,
    slots: &mut HashMap<Option<usize>, u32>,
) -> u32 {
    if let Some(&slot) = slots.get(&material.index()) {
        return slot;
    }

    let slot = mesh.materials.len() as u32;

    mesh.materials.push(mesh_material_from_gltf(material));

    slots.insert(material.index(), slot);

    slot
}

/// Reads a glTF material's flat factors into a [`MeshMaterial`]. The base color
/// and metallic and roughness factors come straight from the metallic-roughness
/// model; the linear base color is sRGB-encoded to the stored `rgba`, while its
/// alpha, which carries no gamma, is scaled directly. Emissive collapses the
/// glTF emissive color to its strongest channel, since Voxel Json models
/// emissive as one strength scaling `rgba`. glTF has no flat occlusion factor
/// (occlusion is a texture), so it defaults to `1` (none).
fn mesh_material_from_gltf(material: &Material) -> MeshMaterial {
    let pbr = material.pbr_metallic_roughness();

    let base = pbr.base_color_factor();

    let rgba = TyLinearRgbaColorF64::new(
        base[0] as f64,
        base[1] as f64,
        base[2] as f64,
        base[3] as f64,
    )
    .to_srgba();

    let emissive = material
        .emissive_factor()
        .into_iter()
        .fold(0.0f32, f32::max) as f64;

    MeshMaterial {
        rgba,
        metallic: pbr.metallic_factor() as f64,
        roughness: pbr.roughness_factor() as f64,
        emissive,
        occlusion: 1.0,
    }
}

/// Transforms `point` by `world` (glTF Y-up) and converts the result to the
/// Voxel Json Z-up axes, a +90 degree rotation about X that sends glTF +Y to
/// +Z, preserving the right-handedness both formats use.
fn world_z_up(world: &TyMatrix4x4F64, point: [f64; 3]) -> TyVector3F64 {
    let [x, y, z] = point;
    let world = world.transform_point(TyVector3F64::new(x, y, z));
    TyVector3F64::new(world.x, -world.z, world.y)
}

#[cfg(test)]
mod tests {
    use crate::{FillMode, MaterialMode, Result, from_gltf_bytes, voxelize_mesh};
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxValue};

    /// A minimal binary glTF (GLB) of an axis-aligned box spanning `[0, sx]`,
    /// `[0, sy]`, `[0, sz]` in glTF Y-up space, indexed triangles. When
    /// `material` is `Some((base_color, metallic, roughness))` the primitive
    /// carries that PBR material; otherwise it uses the glTF default. A
    /// `node_name` names the mesh-bearing node.
    fn box_glb(
        sx: f32,
        sy: f32,
        sz: f32,
        material: Option<([f32; 4], f32, f32)>,
        node_name: Option<&str>,
    ) -> Vec<u8> {
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
        let material_ref = if material.is_some() {
            r#","material":0"#
        } else {
            ""
        };
        let materials = match material {
            Some((color, metallic, roughness)) => format!(
                concat!(
                    r#","materials":[{{"pbrMetallicRoughness":{{"#,
                    r#""baseColorFactor":[{r},{g},{b},{a}],"#,
                    r#""metallicFactor":{metallic},"roughnessFactor":{roughness}}}}}]"#,
                ),
                r = color[0],
                g = color[1],
                b = color[2],
                a = color[3],
                metallic = metallic,
                roughness = roughness,
            ),
            None => String::new(),
        };
        let node_name_attr = match node_name {
            Some(name) => format!(r#","name":"{name}""#),
            None => String::new(),
        };
        let json = format!(
            concat!(
                r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[0]}}],"#,
                r#""nodes":[{{"mesh":0{node_name_attr}}}],"#,
                r#""meshes":[{{"primitives":[{{"attributes":{{"POSITION":0}},"indices":1,"mode":4{material_ref}}}]}}]{materials},"#,
                r#""accessors":[{{"bufferView":0,"componentType":5126,"count":8,"type":"VEC3","#,
                r#""min":[0,0,0],"max":[{sx},{sy},{sz}]}},"#,
                r#"{{"bufferView":1,"componentType":5125,"count":36,"type":"SCALAR"}}],"#,
                r#""bufferViews":[{{"buffer":0,"byteOffset":0,"byteLength":{positions_len}}},"#,
                r#"{{"buffer":0,"byteOffset":{positions_len},"byteLength":{indices_len}}}],"#,
                r#""buffers":[{{"byteLength":{bin_len}}}]}}"#,
            ),
            material_ref = material_ref,
            materials = materials,
            node_name_attr = node_name_attr,
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

    /// Loads `bytes` into a mesh and voxelizes it, the vxl pipeline in one call.
    #[allow(clippy::too_many_arguments)]
    fn voxelize(
        bytes: &[u8],
        counts: TyVector3U32,
        fill_mode: FillMode,
        material_mode: MaterialMode,
        fill_color: Option<[u8; 4]>,
        node_scale: f64,
        name: Option<&str>,
        fallback_name: &str,
    ) -> Result<VoxMain> {
        voxelize_mesh(
            &from_gltf_bytes(bytes)?,
            counts,
            fill_mode,
            material_mode,
            fill_color,
            node_scale,
            name,
            fallback_name,
        )
    }

    /// The rgba hex of the cell a given voxel samples, through the object's one
    /// palette reference.
    fn voxel_hex(state: &VoxMain, position: TyVector3U32) -> String {
        let (_, object) = state.iter_objects().next().unwrap();
        let (reference, _) = object.iter_palette_refs().next().unwrap();
        let (_, palette) = state.iter_palettes().next().unwrap();
        let (rgba, _) = palette
            .iter_attributes()
            .find(|(_, name)| *name == "rgba")
            .unwrap();
        let voxel = object.voxel_id(position).unwrap();
        let cell = object.voxel_cell(voxel, reference).unwrap();
        match palette.cell_value(cell, rgba) {
            Some(VoxValue::Text(hex)) => hex.clone(),
            other => panic!("unexpected rgba value {other:?}"),
        }
    }

    #[test]
    fn extent_converts_gltf_y_up_to_voxel_json_z_up() {
        // A box tall on glTF +Y should be tall on Voxel Json +Z.
        let extent = from_gltf_bytes(&box_glb(1.0, 4.0, 1.0, None, None))
            .unwrap()
            .extent();
        assert!((extent.x - 1.0).abs() < 1e-6, "x extent {}", extent.x);
        assert!((extent.y - 1.0).abs() < 1e-6, "y extent {}", extent.y);
        assert!((extent.z - 4.0).abs() < 1e-6, "z extent {}", extent.z);
    }

    #[test]
    fn flat_paints_the_whole_body_one_color_over_five_attributes() {
        let state = voxelize(
            &box_glb(1.0, 4.0, 1.0, None, None),
            TyVector3U32::new(1, 1, 4),
            FillMode::Solid,
            MaterialMode::Flat,
            Some([255, 0, 0, 255]),
            2.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.object_count(), 1);
        assert_eq!(state.palette_count(), 1);

        let (_, object) = state.iter_objects().next().unwrap();
        assert_eq!(object.bounds(), TyVector3U32::new(1, 1, 4));
        assert_eq!(object.live_count(), 4);

        // Every mode writes the five material attributes; flat is one cell.
        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.cell_count(), 1);
        assert_eq!(
            palette
                .iter_attributes()
                .map(|(_, n)| n)
                .collect::<Vec<_>>(),
            ["rgba", "metallic", "roughness", "emissive", "occlusion"]
        );
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FF0000FF");

        // The root node records the meters-per-voxel scale.
        let root = state.root_hierarchy_nodes()[0];
        assert_eq!(state.hierarchy_node(root).unwrap().transform.scale.z, 2.0);
    }

    #[test]
    fn flat_defaults_fill_none_to_white() {
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, None, None),
            TyVector3U32::new(1, 1, 1),
            FillMode::Solid,
            MaterialMode::Flat,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FFFFFFFF");
    }

    #[test]
    fn per_primitive_reads_the_pbr_factors_and_srgb_encodes_the_base_color() {
        // Linear base color [1, 0.5, 0] sRGB-encodes to #FFBC00; a dropped gamma
        // would give #FF8000 instead.
        let state = voxelize(
            &box_glb(
                1.0,
                1.0,
                1.0,
                Some(([1.0, 0.5, 0.0, 1.0], 0.25, 0.75)),
                None,
            ),
            TyVector3U32::new(1, 1, 1),
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();
        assert_eq!(state.validate(), Ok(()));

        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.cell_count(), 1);
        let cell = palette.iter_cells().next().unwrap();
        let value = |name: &str| {
            let (attribute, _) = palette.iter_attributes().find(|(_, n)| *n == name).unwrap();
            palette.cell_value(cell, attribute).unwrap().clone()
        };
        assert_eq!(value("rgba"), VoxValue::Text("#FFBC00FF".to_owned()));
        assert_eq!(value("metallic"), VoxValue::Number(0.25));
        assert_eq!(value("roughness"), VoxValue::Number(0.75));
        assert_eq!(value("occlusion"), VoxValue::Number(1.0));
    }

    #[test]
    fn per_primitive_fills_a_solid_interior_from_the_nearest_surface() {
        // A one-material solid box: with fill none, the invented interior adopts
        // the surface material, so the whole body is that one color and cell.
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, Some(([1.0, 0.0, 0.0, 1.0], 0.0, 1.0)), None),
            TyVector3U32::new(4, 4, 4),
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            None,
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let (_, object) = state.iter_objects().next().unwrap();
        assert_eq!(object.live_count(), 64);
        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.cell_count(), 1);
        // A deep interior voxel resolved to the surface color.
        assert_eq!(voxel_hex(&state, TyVector3U32::new(2, 2, 2)), "#FF0000FF");
    }

    #[test]
    fn per_primitive_paints_a_solid_interior_with_a_given_fill_color() {
        // With an explicit fill color, the interior is that color, not the
        // sampled surface, so the palette carries both.
        let state = voxelize(
            &box_glb(1.0, 1.0, 1.0, Some(([1.0, 0.0, 0.0, 1.0], 0.0, 1.0)), None),
            TyVector3U32::new(4, 4, 4),
            FillMode::Solid,
            MaterialMode::PerPrimitive,
            Some([0, 0, 255, 255]),
            1.0,
            None,
            "voxelized",
        )
        .unwrap();

        let (_, palette) = state.iter_palettes().next().unwrap();
        assert_eq!(palette.cell_count(), 2);
        assert_eq!(voxel_hex(&state, TyVector3U32::new(0, 0, 0)), "#FF0000FF");
        assert_eq!(voxel_hex(&state, TyVector3U32::new(2, 2, 2)), "#0000FFFF");
    }

    #[test]
    fn rejects_a_grid_past_the_dense_limit() {
        // 1024^3 = 2^30 cells, well past the 2^27 dense limit; this must error
        // before allocating the occupancy grid rather than exhaust memory.
        let result = voxelize(
            &box_glb(1.0, 1.0, 1.0, None, None),
            TyVector3U32::new(1024, 1024, 1024),
            FillMode::Solid,
            MaterialMode::Flat,
            None,
            1.0,
            None,
            "voxelized",
        );
        assert!(result.is_err());
    }

    /// Voxelizes a 1x1x1 box and returns the one object's name.
    fn object_name(bytes: &[u8], name: Option<&str>, fallback: &str) -> String {
        let state = voxelize(
            bytes,
            TyVector3U32::new(1, 1, 1),
            FillMode::Solid,
            MaterialMode::Flat,
            None,
            1.0,
            name,
            fallback,
        )
        .unwrap();
        state.iter_objects().next().unwrap().1.name().to_owned()
    }

    #[test]
    fn names_the_object_from_the_gltf_node_when_not_overridden() {
        let glb = box_glb(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&glb, None, "stem"), "Ship");
    }

    #[test]
    fn an_explicit_name_beats_the_gltf_node() {
        let glb = box_glb(1.0, 1.0, 1.0, None, Some("Ship"));
        assert_eq!(object_name(&glb, Some("Override"), "stem"), "Override");
    }

    #[test]
    fn falls_back_to_the_given_name_when_the_gltf_is_unnamed() {
        let glb = box_glb(1.0, 1.0, 1.0, None, None);
        assert_eq!(object_name(&glb, None, "stem"), "stem");
    }
}
