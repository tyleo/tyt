use crate::{MeshGeometry, MeshMethod};
use branded_id::U32Id;
use ty_math::{TyVector3Ext, TyVector3F32, TyVector3U32};
use voxcore::{BVoxVoxel, VoxObject};

/// Triangulates `object`'s live voxels into a [`MeshGeometry`] using `method`.
///
/// The mesh spans the object's build volume in grid units, Z-up: a live voxel
/// at grid `(x, y, z)` fills the unit cube `[x, x+1] x [y, y+1] x [z, z+1]`.
/// `naive` emits all six faces of every live voxel, `culled` only the faces on
/// a solid-empty boundary, and `greedy` merges coplanar boundary faces into the
/// fewest quads. No hierarchy-node transform is applied; placement is the
/// caller's to add.
pub fn object_to_mesh_geometry(object: &VoxObject, method: MeshMethod) -> MeshGeometry {
    // A constant key merges every coplanar face regardless of material and
    // records no per-vertex material, the fewest-quads pure-geometry mesh.
    mesh_slices(object, method, &|_| 0, false)
}

/// Triangulates `object` with a per-voxel material `key`, so greedy meshing
/// merges only faces whose voxels share a key and every vertex records its key
/// in [`MeshGeometry::material_indices`]. `object_to_mesh_geometry` is this
/// with a constant key and no material tracking.
pub(crate) fn mesh_slices(
    object: &VoxObject,
    method: MeshMethod,
    key: &dyn Fn(U32Id<BVoxVoxel>) -> u32,
    track_materials: bool,
) -> MeshGeometry {
    let bounds = object.bounds().to_array();

    let (cull, merge) = match method {
        MeshMethod::Naive => (false, false),
        MeshMethod::Culled => (true, false),
        MeshMethod::Greedy => (true, true),
    };

    let mut geometry = MeshGeometry::default();

    for d in 0..3 {
        for sign in [-1i32, 1] {
            sweep(
                object,
                bounds,
                d,
                sign,
                cull,
                merge,
                key,
                track_materials,
                &mut geometry,
            );
        }
    }

    geometry
}

/// Sweeps the slices perpendicular to axis `d`, emitting each voxel's face on
/// the `sign` side. `cull` drops a face whose neighbor across it is solid;
/// `merge` fuses the slice's exposed faces into maximal rectangles, splitting
/// where the material key differs.
#[allow(clippy::too_many_arguments)]
fn sweep(
    object: &VoxObject,
    bounds: [u32; 3],
    d: usize,
    sign: i32,
    cull: bool,
    merge: bool,
    key: &dyn Fn(U32Id<BVoxVoxel>) -> u32,
    track_materials: bool,
    geometry: &mut MeshGeometry,
) {
    let u = (d + 1) % 3;
    let v = (d + 2) % 3;
    let w = bounds[u] as usize;
    let h = bounds[v] as usize;

    if w == 0 || h == 0 {
        return;
    }

    for s in 0..bounds[d] {
        // Each exposed face carries its voxel's material key; an unexposed cell
        // is `None`, so merges never cross an empty gap or a key boundary.
        let mut mask = vec![None; w * h];

        for vv in 0..bounds[v] {
            for uu in 0..bounds[u] {
                let mut position = [0i64; 3];
                position[d] = s as i64;
                position[u] = uu as i64;
                position[v] = vv as i64;

                if !live_at(object, position, bounds) {
                    continue;
                }

                if cull {
                    let mut neighbor = position;
                    neighbor[d] = s as i64 + sign as i64;

                    if live_at(object, neighbor, bounds) {
                        continue;
                    }
                }

                let voxel_id = object
                    .voxel_id(TyVector3U32::new(
                        position[0] as u32,
                        position[1] as u32,
                        position[2] as u32,
                    ))
                    .expect("a live voxel is within the grid");

                mask[vv as usize * w + uu as usize] = Some(key(voxel_id));
            }
        }

        if merge {
            for (u0, u1, v0, v1, material) in merge_rects(&mask, w, h) {
                push_face(
                    geometry,
                    d,
                    u,
                    v,
                    sign,
                    s,
                    u0,
                    u1,
                    v0,
                    v1,
                    material,
                    track_materials,
                );
            }
        } else {
            for vv in 0..h {
                for uu in 0..w {
                    if let Some(material) = mask[vv * w + uu] {
                        push_face(
                            geometry,
                            d,
                            u,
                            v,
                            sign,
                            s,
                            uu,
                            uu + 1,
                            vv,
                            vv + 1,
                            material,
                            track_materials,
                        );
                    }
                }
            }
        }
    }
}

/// Whether the grid cell at `position` is a live voxel; out-of-bounds cells and
/// empty cells are both `false`, so a boundary face is exposed.
fn live_at(object: &VoxObject, position: [i64; 3], bounds: [u32; 3]) -> bool {
    if position
        .iter()
        .zip(bounds)
        .any(|(&position_component, bounds_component)| {
            position_component < 0 || position_component >= bounds_component as i64
        })
    {
        return false;
    }

    let position = TyVector3U32::new(position[0] as u32, position[1] as u32, position[2] as u32);

    object
        .voxel_id(position)
        .is_some_and(|voxel_id| object.is_live(voxel_id))
}

/// Greedily fuses a slice `mask` (width `w`, height `h`) into maximal
/// rectangles of one material, each as `(u0, u1, v0, v1, material)` with the
/// upper bounds exclusive. Each set cell belongs to exactly one rectangle, and
/// a rectangle grows only over cells that share its material key.
fn merge_rects(mask: &[Option<u32>], w: usize, h: usize) -> Vec<(usize, usize, usize, usize, u32)> {
    let mut consumed = vec![false; w * h];

    let mut rects = Vec::new();

    for v0 in 0..h {
        for u0 in 0..w {
            let start = v0 * w + u0;
            let Some(material) = mask[start] else {
                continue;
            };
            if consumed[start] {
                continue;
            }

            // Grow the run in +u while the cells share the material and are free.
            let mut width = 1;
            while u0 + width < w {
                let i = v0 * w + u0 + width;
                if mask[i] != Some(material) || consumed[i] {
                    break;
                }
                width += 1;
            }

            // Grow in +v while every cell of the width-wide row matches.
            let mut height = 1;
            'grow: while v0 + height < h {
                for k in 0..width {
                    let i = (v0 + height) * w + u0 + k;
                    if mask[i] != Some(material) || consumed[i] {
                        break 'grow;
                    }
                }
                height += 1;
            }

            for dy in 0..height {
                for dx in 0..width {
                    consumed[(v0 + dy) * w + u0 + dx] = true;
                }
            }

            rects.push((u0, u0 + width, v0, v0 + height, material));
        }
    }
    rects
}

/// Appends the quad on axis `d`'s `sign` face of slice `s`, spanning `u` in
/// `[u0, u1]` and `v` in `[v0, v1]`, wound counter-clockwise outward. When
/// `track_materials` is set, every vertex records `material`.
#[allow(clippy::too_many_arguments)]
fn push_face(
    geometry: &mut MeshGeometry,
    d: usize,
    u: usize,
    v: usize,
    sign: i32,
    s: u32,
    u0: usize,
    u1: usize,
    v0: usize,
    v1: usize,
    material: u32,
    track_materials: bool,
) {
    // The +side face sits one unit past the slice along `d`, the -side on it.
    let plane = s as f32 + if sign > 0 { 1.0 } else { 0.0 };

    let corner = |along_u: f32, along_v: f32| {
        let mut point = [0f32; 3];
        point[d] = plane;
        point[u] = along_u;
        point[v] = along_v;
        TyVector3F32::from_array(point)
    };

    let (u0, u1, v0, v1) = (u0 as f32, u1 as f32, v0 as f32, v1 as f32);

    let p00 = corner(u0, v0);
    let p10 = corner(u1, v0);
    let p11 = corner(u1, v1);
    let p01 = corner(u0, v1);

    let mut normal = [0f32; 3];
    normal[d] = sign as f32;
    let normal = TyVector3F32::from_array(normal);

    // The (u, v) axes may be oriented either way about `d`, so wind the corners
    // by whether the u-then-v corner cross points along the outward normal.
    let outward = TyVector3F32::triangle_normal(p00, p10, p01).dot(normal) >= 0.0;

    let corners = if outward {
        [p00, p10, p11, p01]
    } else {
        [p00, p01, p11, p10]
    };

    let base = geometry.positions.len() as u32;

    for corner in corners {
        geometry.positions.push(corner);
        geometry.normals.push(normal);

        if track_materials {
            geometry.material_indices.push(material);
        }
    }

    geometry
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use crate::{MeshMethod, mesh_slices, object_to_mesh_geometry};
    use ty_math::{TyVector3Ext, TyVector3F32, TyVector3U32};
    use voxcore::VoxObject;

    /// A build-volume object of `bounds` with `live` cells filled, no palettes.
    fn object(bounds: [u32; 3], live: &[[u32; 3]]) -> VoxObject {
        let mut object = VoxObject::new(
            "m".to_owned(),
            TyVector3U32::new(bounds[0], bounds[1], bounds[2]),
        )
        .unwrap();
        for &[x, y, z] in live {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        object
    }

    #[test]
    fn naive_emits_all_six_faces_per_voxel() {
        let object = object([1, 1, 1], &[[0, 0, 0]]);
        let mesh = object_to_mesh_geometry(&object, MeshMethod::Naive);
        assert_eq!(mesh.quad_count(), 6);
        assert_eq!(mesh.triangle_count(), 12);
        assert_eq!(mesh.vertex_count(), 24);
    }

    #[test]
    fn culled_drops_the_shared_interior_faces() {
        // Two adjacent voxels: 12 faces total, the shared pair is interior.
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object_to_mesh_geometry(&object, MeshMethod::Naive).quad_count(),
            12
        );
        assert_eq!(
            object_to_mesh_geometry(&object, MeshMethod::Culled).quad_count(),
            10
        );
    }

    #[test]
    fn greedy_merges_a_two_voxel_bar_into_a_box() {
        // A 2x1x1 box exposes one rectangle per face.
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object_to_mesh_geometry(&object, MeshMethod::Greedy).quad_count(),
            6
        );
    }

    #[test]
    fn greedy_collapses_a_solid_slab() {
        // 3x3x1 solid: culled = 2*(3*3 + 3*1 + 1*3) = 30 quads; greedy = 6.
        let live: Vec<[u32; 3]> = (0..3)
            .flat_map(|x| (0..3).map(move |y| [x, y, 0]))
            .collect();
        let object = object([3, 3, 1], &live);
        assert_eq!(
            object_to_mesh_geometry(&object, MeshMethod::Culled).quad_count(),
            30
        );
        assert_eq!(
            object_to_mesh_geometry(&object, MeshMethod::Greedy).quad_count(),
            6
        );
    }

    #[test]
    fn material_keys_split_a_merged_run() {
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);

        // A uniform key merges the bar into a box: six quads, no per-vertex
        // materials.
        let pure = object_to_mesh_geometry(&object, MeshMethod::Greedy);
        assert_eq!(pure.quad_count(), 6);
        assert!(pure.material_indices.is_empty());

        // Two materials along x (voxel id 0 vs 1) split every face that spanned
        // both voxels: the two end caps stay, the four side faces each split in
        // two, for ten quads.
        let keyed = mesh_slices(
            &object,
            MeshMethod::Greedy,
            &|voxel_id| voxel_id.to_u32(),
            true,
        );
        assert_eq!(keyed.quad_count(), 10);
        assert_eq!(keyed.material_indices.len(), keyed.vertex_count());
        assert!(
            keyed
                .material_indices
                .iter()
                .all(|&material_index| material_index == 0 || material_index == 1)
        );
    }

    #[test]
    fn a_uniform_key_merges_yet_records_materials() {
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);

        // One material still merges into a box, but every vertex records it.
        let keyed = mesh_slices(&object, MeshMethod::Greedy, &|_| 0, true);
        assert_eq!(keyed.quad_count(), 6);
        assert_eq!(keyed.material_indices.len(), keyed.vertex_count());
        assert!(
            keyed
                .material_indices
                .iter()
                .all(|&material_index| material_index == 0)
        );
    }

    #[test]
    fn single_voxel_spans_the_unit_cube() {
        let object = object([1, 1, 1], &[[0, 0, 0]]);
        let mesh = object_to_mesh_geometry(&object, MeshMethod::Culled);
        for point in &mesh.positions {
            assert!(
                point.to_array().iter().all(|&c| c == 0.0 || c == 1.0),
                "corner {point:?}"
            );
        }
    }

    #[test]
    fn every_triangle_winds_outward() {
        // A 2x2x2 solid cube exercises all six face directions under greedy.
        let live: Vec<[u32; 3]> = (0..2)
            .flat_map(|x| (0..2).flat_map(move |y| (0..2).map(move |z| [x, y, z])))
            .collect();
        let object = object([2, 2, 2], &live);
        let mesh = object_to_mesh_geometry(&object, MeshMethod::Greedy);
        assert_eq!(mesh.quad_count(), 6);

        for triangle in mesh.indices.chunks_exact(3) {
            let corner = |i: u32| mesh.positions[i as usize];
            let (p0, p1, p2) = (
                corner(triangle[0]),
                corner(triangle[1]),
                corner(triangle[2]),
            );
            let stored = mesh.normals[triangle[0] as usize];
            assert!(
                TyVector3F32::triangle_normal(p0, p1, p2).dot(stored) > 0.0,
                "triangle winds inward"
            );
        }
    }
}
