use crate::operations::mesh::{MeshGeometry, is_solid};
use vox_value_language::{Components, Dimension, Domain, Value};
use voxcore::VoxObject;

/// Each face corner's occlusion from the voxels meeting there, a corner `f32`
/// vec1 array in `[0, 1]` with `1` fully open. Each of the three cells beside
/// the corner in the layer the face looks into closes a third. Both cells
/// along the face's edges together close it fully. So does a solid cell over
/// the face, which only `naive` emits.
pub(crate) fn compute_occlusion(object: &VoxObject, geometry: &MeshGeometry) -> Value {
    let mut components = Vec::with_capacity(geometry.positions.len());

    for quad in 0..geometry.quad_count() {
        let corners: Vec<[f32; 3]> = geometry.positions[quad * 4..quad * 4 + 4]
            .iter()
            .map(|position| position.to_array())
            .collect();

        let normal = geometry.normals[quad * 4].to_array();

        let axis = (0..3)
            .find(|&axis| normal[axis] != 0.0)
            .expect("a face normal lies along one axis");

        let [first, second] = match axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        };

        let center = |axis: usize| corners.iter().map(|corner| corner[axis]).sum::<f32>() / 4.0;
        let centers = [center(first), center(second)];

        for corner in &corners {
            let mut cell = [0i64; 3];
            cell[axis] = if normal[axis] > 0.0 {
                corner[axis] as i64
            } else {
                corner[axis] as i64 - 1
            };

            // Along each tangent axis, the cell under the face and the cell
            // beside the corner outside it.
            let along = |tangent: usize, center: f32| {
                let at = corner[tangent] as i64;
                if corner[tangent] < center {
                    (at, at - 1)
                } else {
                    (at - 1, at)
                }
            };
            let (under_first, beside_first) = along(first, centers[0]);
            let (under_second, beside_second) = along(second, centers[1]);

            let solid = |first_at: i64, second_at: i64| {
                let mut cell = cell;
                cell[first] = first_at;
                cell[second] = second_at;
                is_solid(object, cell)
            };

            let over = solid(under_first, under_second);
            let side_first = solid(beside_first, under_second);
            let side_second = solid(under_first, beside_second);
            let diagonal = solid(beside_first, beside_second);

            let open = if over || (side_first && side_second) {
                0
            } else {
                3 - u8::from(side_first) - u8::from(side_second) - u8::from(diagonal)
            };

            components.push(f32::from(open) / 3.0);
        }
    }

    Value::new(Domain::Corner, Dimension::Vec1, Components::F32(components))
        .expect("one component per corner fills a vec1 array")
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{Method, compute_occlusion, object_to_mesh_geometry};
    use ty_math::TyVector3U32;
    use vox_value_language::Components;
    use voxcore::VoxObject;

    fn object(live: &[[u32; 3]]) -> VoxObject {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 3, 3)).unwrap();
        for &[x, y, z] in live {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        object
    }

    /// The occlusion at each corner of the quads facing `normal`, as
    /// `(corner position, occlusion)`.
    fn facing(object: &VoxObject, method: Method, normal: [f32; 3]) -> Vec<([f32; 3], f32)> {
        let geometry = object_to_mesh_geometry(object, method);
        let value = compute_occlusion(object, &geometry);
        let Components::F32(occlusion) = value.components() else {
            panic!("occlusion is f32");
        };

        (0..geometry.quad_count())
            .filter(|&quad| geometry.normals[quad * 4].to_array() == normal)
            .flat_map(|quad| {
                (quad * 4..quad * 4 + 4)
                    .map(|corner| (geometry.positions[corner].to_array(), occlusion[corner]))
            })
            .collect()
    }

    #[test]
    fn a_lone_voxel_is_open_at_every_corner() {
        let object = object(&[[1, 1, 1]]);
        let geometry = object_to_mesh_geometry(&object, Method::Culled);

        let value = compute_occlusion(&object, &geometry);

        assert_eq!(value.entries(), 24);
        assert_eq!(value.components(), &Components::F32(vec![1.0; 24]));
    }

    #[test]
    fn a_neighbor_beside_a_corner_closes_a_third_and_two_close_it() {
        // A step: the top of (0,0,0) meets (1,0,1) along its x = 1 edge.
        let step = object(&[[0, 0, 0], [1, 0, 0], [1, 0, 1]]);
        for (position, occlusion) in facing(&step, Method::Culled, [0.0, 0.0, 1.0]) {
            if position[2] != 1.0 {
                continue;
            }
            let expected = if position[0] == 1.0 { 2.0 / 3.0 } else { 1.0 };
            assert_eq!(occlusion, expected, "{position:?}");
        }

        // An inner corner: the top of (0,0,0) meets both (1,0,1) and (0,1,1),
        // and the diagonal (1,1,1) is empty.
        let inner = object(&[[0, 0, 0], [1, 0, 1], [0, 1, 1]]);
        let corners = facing(&inner, Method::Culled, [0.0, 0.0, 1.0]);
        let (_, occlusion) = corners
            .iter()
            .find(|(position, _)| *position == [1.0, 1.0, 1.0])
            .unwrap();
        assert_eq!(*occlusion, 0.0);
    }

    #[test]
    fn a_face_under_a_solid_cell_is_closed() {
        // Under naive the shared face between two voxels still emits.
        let pair = object(&[[0, 0, 0], [1, 0, 0]]);
        let corners = facing(&pair, Method::Naive, [1.0, 0.0, 0.0]);
        let buried: Vec<f32> = corners
            .iter()
            .filter(|(position, _)| position[0] == 1.0)
            .map(|(_, occlusion)| *occlusion)
            .collect();
        assert_eq!(buried, [0.0; 4]);
    }
}
