use crate::{SurfaceGrid, SurfaceMesh, corner_occlusion};

/// The [`corner_occlusion`] of every quad of `mesh`, one value per vertex
/// in vertex order.
pub fn mesh_occlusion<G: SurfaceGrid>(grid: &G, mesh: &SurfaceMesh<G::Cell>) -> Vec<f64> {
    mesh.spans
        .iter()
        .flat_map(|span| corner_occlusion(grid, span))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{SurfaceMethod, mesh_grid, mesh_occlusion, test_utilities::live_object};
    use voxcore::VoxObject;

    /// The occlusion at each vertex of the quads facing `normal`, as
    /// `(vertex position, occlusion)`.
    fn facing(object: &VoxObject, method: SurfaceMethod, normal: [f32; 3]) -> Vec<([f32; 3], f64)> {
        let mesh = mesh_grid(object, method);
        let occlusion = mesh_occlusion(object, &mesh);

        (0..mesh.quad_count())
            .filter(|&quad| mesh.normals[quad * 4].to_array() == normal)
            .flat_map(|quad| {
                (quad * 4..quad * 4 + 4)
                    .map(|vertex| (mesh.positions[vertex].to_array(), occlusion[vertex]))
            })
            .collect()
    }

    #[test]
    fn a_lone_voxel_is_open_at_every_vertex() {
        let object = live_object([3, 3, 3], &[[1, 1, 1]]);
        let mesh = mesh_grid(&object, SurfaceMethod::Culled);

        assert_eq!(mesh_occlusion(&object, &mesh), vec![1.0; 24]);
    }

    #[test]
    fn each_vertex_reads_the_occlusion_at_its_position() {
        // A step: the top of (0,0,0) meets (1,0,1) along its x = 1 edge.
        let step = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 0], [1, 0, 1]]);
        let tops = facing(&step, SurfaceMethod::Culled, [0.0, 0.0, 1.0]);
        assert_eq!(tops.len(), 8);
        for (position, occlusion) in tops {
            if position[2] != 1.0 {
                continue;
            }
            let expected = if position[0] == 1.0 { 2.0 / 3.0 } else { 1.0 };
            assert_eq!(occlusion, expected, "{position:?}");
        }
    }

    #[test]
    fn a_face_under_a_solid_cell_is_closed() {
        // Under naive the shared face between two voxels still emits.
        let pair = live_object([3, 3, 3], &[[0, 0, 0], [1, 0, 0]]);
        let buried: Vec<f64> = facing(&pair, SurfaceMethod::Naive, [1.0, 0.0, 0.0])
            .iter()
            .filter(|(position, _)| position[0] == 1.0)
            .map(|(_, occlusion)| *occlusion)
            .collect();
        assert_eq!(buried, [0.0; 4]);
    }
}
