use crate::operations::mesh::{FaceSpan, MeshGeometry, Method, is_solid};
use branded_id::U32Id;
use ty_math::{TyVector3Ext, TyVector3F32, TyVector3U32};
use voxcore::{BVoxVoxel, VoxObject};

/// Triangulates `object`'s live voxels into a [`MeshGeometry`] using `method`.
///
/// The mesh spans the object's build volume in grid units on the grid's axes:
/// a live voxel at grid `(x, y, z)` fills the unit cube
/// `[x, x+1] x [y, y+1] x [z, z+1]`.
/// `naive` emits all six faces of every live voxel, `culled` only the faces on
/// a solid-empty boundary, and `greedy` merges coplanar boundary faces into the
/// fewest quads. No hierarchy-node transform is applied; placement is the
/// caller's to add.
pub(crate) fn object_to_mesh_geometry(object: &VoxObject, method: Method) -> MeshGeometry {
    // A constant key merges every coplanar face regardless of material and
    // records no per-vertex material, the fewest-quads pure-geometry mesh.
    mesh_slices(object, method, &|_| 0, &|_| true, false)
}

/// Triangulates `object` with a per-voxel `key`, so greedy meshing merges
/// only faces whose voxels share a key and grows a span only while
/// `span_fits` accepts it. When `track_materials` is set, every vertex
/// records its key in [`MeshGeometry::material_indices`].
pub(crate) fn mesh_slices(
    object: &VoxObject,
    method: Method,
    key: &dyn Fn(U32Id<BVoxVoxel>) -> u32,
    span_fits: &dyn Fn(&FaceSpan) -> bool,
    track_materials: bool,
) -> MeshGeometry {
    let bounds = object.bounds().to_array();

    let (cull, merge) = match method {
        Method::Naive => (false, false),
        Method::Culled => (true, false),
        Method::Greedy => (true, true),
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
                span_fits,
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
/// where the key differs or `span_fits` refuses.
#[allow(clippy::too_many_arguments)]
fn sweep(
    object: &VoxObject,
    bounds: [u32; 3],
    d: usize,
    sign: i32,
    cull: bool,
    merge: bool,
    key: &dyn Fn(U32Id<BVoxVoxel>) -> u32,
    span_fits: &dyn Fn(&FaceSpan) -> bool,
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
        let span = |u0, u1, v0, v1| FaceSpan {
            d,
            sign,
            s,
            u0,
            u1,
            v0,
            v1,
        };

        // Each exposed face carries its voxel's key; an unexposed cell is
        // `None`, so merges never cross an empty gap or a key boundary.
        let mut mask = vec![None; w * h];

        for vv in 0..h {
            for uu in 0..w {
                let position = span(uu, uu + 1, vv, vv + 1).cell(uu, vv);
                let signed = position.map(i64::from);

                if !is_solid(object, signed) {
                    continue;
                }

                if cull {
                    let mut neighbor = signed;
                    neighbor[d] += i64::from(sign);

                    if is_solid(object, neighbor) {
                        continue;
                    }
                }

                let voxel_id = object
                    .voxel_id(TyVector3U32::from_array(position))
                    .expect("a live voxel is within the grid");

                mask[vv * w + uu] = Some(key(voxel_id));
            }
        }

        if merge {
            let fits = |u0, u1, v0, v1| span_fits(&span(u0, u1, v0, v1));

            for (u0, u1, v0, v1, material) in merge_rects(&mask, w, h, &fits) {
                push_face(
                    object,
                    geometry,
                    &span(u0, u1, v0, v1),
                    material,
                    track_materials,
                );
            }
        } else {
            for vv in 0..h {
                for uu in 0..w {
                    if let Some(material) = mask[vv * w + uu] {
                        push_face(
                            object,
                            geometry,
                            &span(uu, uu + 1, vv, vv + 1),
                            material,
                            track_materials,
                        );
                    }
                }
            }
        }
    }
}

/// Greedily fuses a slice `mask` (width `w`, height `h`) into maximal
/// rectangles of one key, each as `(u0, u1, v0, v1, key)` with the upper
/// bounds exclusive. Each set cell belongs to exactly one rectangle, and a
/// rectangle grows only over cells that share its key and only while `fits`
/// accepts the grown rectangle.
fn merge_rects(
    mask: &[Option<u32>],
    w: usize,
    h: usize,
    fits: &dyn Fn(usize, usize, usize, usize) -> bool,
) -> Vec<(usize, usize, usize, usize, u32)> {
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

            // Grow the run in +u while the cells share the key and are free.
            let mut width = 1;
            while u0 + width < w {
                let i = v0 * w + u0 + width;
                if mask[i] != Some(material) || consumed[i] || !fits(u0, u0 + width + 1, v0, v0 + 1)
                {
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
                if !fits(u0, u0 + width, v0, v0 + height + 1) {
                    break;
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

/// Appends the quad over `span`, wound counter-clockwise outward, and records
/// the voxels it covers. When `track_materials` is set, every vertex records
/// `material`.
fn push_face(
    object: &VoxObject,
    geometry: &mut MeshGeometry,
    span: &FaceSpan,
    material: u32,
    track_materials: bool,
) {
    let voxel_ids = span
        .cells()
        .map(|position| {
            object
                .voxel_id(TyVector3U32::from_array(position))
                .expect("a quad covers cells within the grid")
        })
        .collect();

    geometry.face_voxel_ids.push(voxel_ids);

    let (u0, u1) = (span.u0 as f32, span.u1 as f32);
    let (v0, v1) = (span.v0 as f32, span.v1 as f32);

    let p00 = span.corner(u0, v0);
    let p10 = span.corner(u1, v0);
    let p11 = span.corner(u1, v1);
    let p01 = span.corner(u0, v1);

    let normal = span.normal();

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
    use crate::operations::mesh::{FaceSpan, Method, mesh_slices, object_to_mesh_geometry};
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
        let mesh = object_to_mesh_geometry(&object, Method::Naive);
        assert_eq!(mesh.quad_count(), 6);
        assert_eq!(mesh.indices.len(), 36);
        assert_eq!(mesh.positions.len(), 24);
    }

    #[test]
    fn culled_drops_the_shared_interior_faces() {
        // Two adjacent voxels: 12 faces total, the shared pair is interior.
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Naive).quad_count(),
            12
        );
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Culled).quad_count(),
            10
        );
    }

    #[test]
    fn greedy_merges_a_two_voxel_bar_into_a_box() {
        // A 2x1x1 box exposes one rectangle per face.
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Greedy).quad_count(),
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
            object_to_mesh_geometry(&object, Method::Culled).quad_count(),
            30
        );
        assert_eq!(
            object_to_mesh_geometry(&object, Method::Greedy).quad_count(),
            6
        );
    }

    #[test]
    fn material_keys_split_a_merged_run() {
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);

        // A uniform key merges the bar into a box: six quads, no per-vertex
        // materials.
        let pure = object_to_mesh_geometry(&object, Method::Greedy);
        assert_eq!(pure.quad_count(), 6);
        assert!(pure.material_indices.is_empty());

        // Two materials along x (voxel id 0 vs 1) split every face that spanned
        // both voxels: the two end caps stay, the four side faces each split in
        // two, for ten quads.
        let keyed = mesh_slices(
            &object,
            Method::Greedy,
            &|voxel_id| voxel_id.to_u32(),
            &|_| true,
            true,
        );
        assert_eq!(keyed.quad_count(), 10);
        assert_eq!(keyed.material_indices.len(), keyed.positions.len());
        assert!(
            keyed
                .material_indices
                .iter()
                .all(|&material_index| material_index == 0 || material_index == 1)
        );
    }

    #[test]
    fn a_refused_span_stops_growing_in_either_direction() {
        // A 3x3 slab whose spans may cover two cells at most.
        let live: Vec<[u32; 3]> = (0..3)
            .flat_map(|x| (0..3).map(move |y| [x, y, 0]))
            .collect();
        let object = object([3, 3, 1], &live);

        let at_most_two = |span: &FaceSpan| (span.u1 - span.u0) * (span.v1 - span.v0) <= 2;
        let geometry = mesh_slices(&object, Method::Greedy, &|_| 0, &at_most_two, false);

        // Each 3x3 face splits into four 2x1 runs and one lone cell; each
        // 3x1 side stays a 2x1 run and a 1x1.
        assert_eq!(geometry.quad_count(), 2 * 5 + 4 * 2);
        assert!(
            geometry
                .face_voxel_ids
                .iter()
                .all(|voxel_ids| voxel_ids.len() <= 2)
        );
    }

    #[test]
    fn a_uniform_key_merges_yet_records_materials() {
        let object = object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);

        // One material still merges into a box, but every vertex records it.
        let keyed = mesh_slices(&object, Method::Greedy, &|_| 0, &|_| true, true);
        assert_eq!(keyed.quad_count(), 6);
        assert_eq!(keyed.material_indices.len(), keyed.positions.len());
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
        let mesh = object_to_mesh_geometry(&object, Method::Culled);
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
        let mesh = object_to_mesh_geometry(&object, Method::Greedy);
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
