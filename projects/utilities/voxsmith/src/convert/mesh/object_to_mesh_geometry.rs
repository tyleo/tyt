use crate::{MeshGeometry, MeshMethod};
use ty_math::{TyVector3F32, TyVector3U32};
use voxcore::VoxObject;

/// Triangulates `object`'s live voxels into a [`MeshGeometry`] using `method`.
///
/// The mesh spans the object's build volume in grid units, Z-up: a live voxel
/// at grid `(x, y, z)` fills the unit cube `[x, x+1] x [y, y+1] x [z, z+1]`.
/// `naive` emits all six faces of every live voxel, `culled` only the faces on
/// a solid-empty boundary, and `greedy` merges coplanar boundary faces into the
/// fewest quads. No hierarchy-node transform is applied; placement is the
/// caller's to add.
pub fn object_to_mesh_geometry(object: &VoxObject, method: MeshMethod) -> MeshGeometry {
    let bounds = object.bounds();

    let bounds = [bounds.x, bounds.y, bounds.z];

    let (cull, merge) = match method {
        MeshMethod::Naive => (false, false),
        MeshMethod::Culled => (true, false),
        MeshMethod::Greedy => (true, true),
    };

    let mut geometry = MeshGeometry::default();

    for d in 0..3 {
        for sign in [-1i32, 1] {
            sweep(object, bounds, d, sign, cull, merge, &mut geometry);
        }
    }

    geometry
}

/// Sweeps the slices perpendicular to axis `d`, emitting each voxel's face on
/// the `sign` side. `cull` drops a face whose neighbor across it is solid;
/// `merge` fuses the slice's exposed faces into maximal rectangles.
#[allow(clippy::too_many_arguments)]
fn sweep(
    object: &VoxObject,
    bounds: [u32; 3],
    d: usize,
    sign: i32,
    cull: bool,
    merge: bool,
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
        // The exposed-face mask over the slice's (u, v) plane.
        let mut mask = vec![false; w * h];

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

                mask[vv as usize * w + uu as usize] = true;
            }
        }

        if merge {
            for (u0, u1, v0, v1) in merge_rects(&mask, w, h) {
                push_face(geometry, d, u, v, sign, s, u0, u1, v0, v1);
            }
        } else {
            for vv in 0..h {
                for uu in 0..w {
                    if mask[vv * w + uu] {
                        push_face(geometry, d, u, v, sign, s, uu, uu + 1, vv, vv + 1);
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
        .is_some_and(|id| object.is_live(id))
}

/// Greedily fuses a slice `mask` (width `w`, height `h`) into maximal
/// rectangles, each as `(u0, u1, v0, v1)` with the upper bounds exclusive. Each
/// true cell belongs to exactly one rectangle.
fn merge_rects(mask: &[bool], w: usize, h: usize) -> Vec<(usize, usize, usize, usize)> {
    let mut consumed = vec![false; w * h];

    let mut rects = Vec::new();

    for v0 in 0..h {
        for u0 in 0..w {
            let start = v0 * w + u0;
            if !mask[start] || consumed[start] {
                continue;
            }

            // Grow the run in +u while the cells stay set and free.
            let mut width = 1;
            while u0 + width < w {
                let i = v0 * w + u0 + width;
                if !mask[i] || consumed[i] {
                    break;
                }
                width += 1;
            }

            // Grow in +v while every cell of the width-wide row matches.
            let mut height = 1;
            'grow: while v0 + height < h {
                for k in 0..width {
                    let i = (v0 + height) * w + u0 + k;
                    if !mask[i] || consumed[i] {
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

            rects.push((u0, u0 + width, v0, v0 + height));
        }
    }
    rects
}

/// Appends the quad on axis `d`'s `sign` face of slice `s`, spanning `u` in
/// `[u0, u1]` and `v` in `[v0, v1]`, wound counter-clockwise outward.
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
    let outward = (p10 - p00).cross(&(p01 - p00)).dot(&normal) >= 0.0;

    let corners = if outward {
        [p00, p10, p11, p01]
    } else {
        [p00, p01, p11, p10]
    };

    let base = geometry.positions.len() as u32;

    for corner in corners {
        geometry.positions.push(corner);
        geometry.normals.push(normal);
    }

    geometry
        .indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use crate::{MeshMethod, object_to_mesh_geometry};
    use ty_math::TyVector3U32;
    use voxcore::VoxObject;

    /// A build-volume object of `bounds` with `live` cells filled, no palettes.
    fn object(bounds: [u32; 3], live: &[[u32; 3]]) -> VoxObject {
        let mut object = VoxObject::new(
            "m".to_owned(),
            TyVector3U32::new(bounds[0], bounds[1], bounds[2]),
        )
        .unwrap();
        for &[x, y, z] in live {
            let id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
            object.retain_voxel(id, &[]).unwrap();
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
                (p1 - p0).cross(&(p2 - p0)).dot(&stored) > 0.0,
                "triangle winds inward"
            );
        }
    }
}
