use crate::{
    Result,
    operations::mesh::{ArrayDomain, AtlasLayout, MeshGeometry, Swatches, TextureShape},
};
use ty_math::TyVector2F64;

/// The atlases over one geometry, each laid out on demand, and each face's
/// cell on them.
pub(crate) struct Atlases<'a> {
    shape: TextureShape,
    swatches: &'a Swatches<'a>,
    geometry: &'a MeshGeometry,
}

impl<'a> Atlases<'a> {
    pub(crate) fn new(
        shape: TextureShape,
        swatches: &'a Swatches<'a>,
        geometry: &'a MeshGeometry,
    ) -> Self {
        Atlases {
            shape,
            swatches,
            geometry,
        }
    }

    /// The `domain` atlas's layout.
    pub(crate) fn layout(&self, domain: ArrayDomain) -> Result<AtlasLayout> {
        let count = match domain {
            ArrayDomain::Corner | ArrayDomain::Face => self.geometry.quad_count(),
            ArrayDomain::Swatch => self.swatches.count(),
            ArrayDomain::Voxel => self.swatches.voxel_swatch_ids().len(),
        };

        AtlasLayout::shape(domain, count, self.shape)
    }

    /// The cell face `face` occupies on the `domain` atlas. The merge rules
    /// keep a face on the swatch atlas to one swatch and a face on the voxel
    /// atlas to one voxel.
    pub(crate) fn cell(&self, domain: ArrayDomain, face: usize) -> usize {
        let voxel_ids = &self.geometry.face_voxel_ids[face];

        match domain {
            ArrayDomain::Corner | ArrayDomain::Face => face,

            ArrayDomain::Swatch => {
                let mut swatch_ids = voxel_ids.iter().map(|&voxel_id| {
                    self.swatches.voxel_swatch_ids()
                        [self.swatches.voxel_entry_id(voxel_id).to_usize_id()]
                });

                let swatch_id = swatch_ids.next().expect("a face covers a voxel");

                assert!(
                    swatch_ids.all(|other| other == swatch_id),
                    "a face on the swatch atlas covers one swatch"
                );

                swatch_id.to_usize_id().to_usize()
            }

            ArrayDomain::Voxel => {
                let [voxel_id] = voxel_ids.as_slice() else {
                    unreachable!("a face on the voxel atlas covers one voxel");
                };

                self.swatches
                    .voxel_entry_id(*voxel_id)
                    .to_usize_id()
                    .to_usize()
            }
        }
    }

    /// One UV per vertex on the `domain` atlas.
    pub(crate) fn uvs(&self, domain: ArrayDomain) -> Result<Vec<TyVector2F64>> {
        let layout = self.layout(domain)?;

        Ok((0..self.geometry.quad_count())
            .flat_map(|face| {
                let cell = self.cell(domain, face);
                (0..4).map(move |corner| layout.uv(cell, corner))
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{
        ArrayDomain, Atlases, Method, Swatches, TextureShape, object_to_mesh_geometry,
    };
    use ty_math::{TyVector2F64, TyVector3U32};
    use voxcore::{VoxMain, VoxObject};

    /// A 2x1x1 bar of two live voxels with no layers.
    fn bar() -> VoxObject {
        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }
        object
    }

    #[test]
    fn each_atlas_counts_its_own_cells_and_places_every_vertex() {
        let main: VoxMain = VoxMain::default();
        let object = bar();
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let culled = object_to_mesh_geometry(&object, Method::Culled);
        let atlases = Atlases::new(TextureShape::Pot, &swatches, &culled);

        // One swatch, two voxels, ten faces, and ten corner blocks.
        for (domain, width, height) in [
            (ArrayDomain::Corner, 8, 8),
            (ArrayDomain::Face, 4, 4),
            (ArrayDomain::Swatch, 1, 1),
            (ArrayDomain::Voxel, 2, 2),
        ] {
            let layout = atlases.layout(domain).unwrap();
            assert_eq!(
                (layout.width(), layout.height()),
                (width, height),
                "{domain}"
            );
            assert_eq!(atlases.uvs(domain).unwrap().len(), 40, "{domain}");
        }

        // Every face of the one swatch reads its one texel.
        assert!(
            atlases
                .uvs(ArrayDomain::Swatch)
                .unwrap()
                .iter()
                .all(|&uv| uv == TyVector2F64::new(0.5, 0.5))
        );

        // The last face sits at cell 9 of the face atlas, and its corners
        // go around cell 9's block on the corner atlas.
        assert_eq!(
            atlases.uvs(ArrayDomain::Face).unwrap()[36],
            TyVector2F64::new(0.375, 0.625)
        );
        assert_eq!(
            &atlases.uvs(ArrayDomain::Corner).unwrap()[36..],
            [
                TyVector2F64::new(2.5 / 8.0, 4.5 / 8.0),
                TyVector2F64::new(3.5 / 8.0, 4.5 / 8.0),
                TyVector2F64::new(3.5 / 8.0, 5.5 / 8.0),
                TyVector2F64::new(2.5 / 8.0, 5.5 / 8.0),
            ]
        );
    }

    #[test]
    fn a_face_reads_the_texel_of_the_voxel_it_covers() {
        let main: VoxMain = VoxMain::default();
        let object = bar();
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let culled = object_to_mesh_geometry(&object, Method::Culled);
        let atlases = Atlases::new(TextureShape::Line, &swatches, &culled);

        let uvs = atlases.uvs(ArrayDomain::Voxel).unwrap();

        let mut cells: Vec<f64> = uvs.iter().map(|uv| uv.x).collect();
        cells.sort_by(f64::total_cmp);
        cells.dedup();
        assert_eq!(cells, [0.25, 0.75]);
        assert_eq!(
            uvs.iter().filter(|uv| uv.x == 0.25).count(),
            20,
            "each voxel's five faces read its texel"
        );
    }
}
