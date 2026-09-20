use crate::operations::mesh::{MeshGeometry, Swatches};
use vox_value_language::Groupings;

/// The tables the reductions and climbs walk: each voxel entry's swatch and
/// each face's voxel pieces.
pub(crate) fn groupings_of(swatches: &Swatches<'_>, geometry: &MeshGeometry) -> Groupings {
    Groupings {
        voxel_swatches: swatches.voxel_swatch_ids().clone(),
        face_voxels: geometry
            .face_voxel_ids
            .iter()
            .map(|voxel_ids| {
                voxel_ids
                    .iter()
                    .map(|&voxel_id| swatches.voxel_entry_id(voxel_id))
                    .collect()
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{Method, Swatches, groupings_of, object_to_mesh_geometry};
    use ty_math::TyVector3U32;
    use voxcore::{VoxMain, VoxObject};

    #[test]
    fn a_merged_face_lists_every_voxel_it_covers() {
        let main: VoxMain = VoxMain::default();
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let geometry = object_to_mesh_geometry(&object, Method::Greedy);

        let groupings = groupings_of(&swatches, &geometry);

        assert_eq!(groupings.voxel_swatches.len(), 2);
        assert_eq!(groupings.face_voxels.len(), 6);
        let mut pieces: Vec<usize> = groupings
            .face_voxels
            .iter()
            .map(|voxel_ids| voxel_ids.len())
            .collect();
        pieces.sort_unstable();
        assert_eq!(pieces, [1, 1, 2, 2, 2, 2]);
    }
}
