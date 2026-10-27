use crate::{
    Error, Result,
    operations::object::{
        Computation, ComputedBinding, MeshElement, MeshGeometry, SwatchGrid, Swatches,
    },
    utilities::property_value,
};
use branded_id::IdRange;
use std::collections::{BTreeSet, HashMap};
use vox_value_language::{
    Components, Dimension, Domain, Groupings, Scalar, Type, TypeEnvironment, Value,
    ValueEnvironment,
};
use voxcore::VoxObject;
use voxsurface::mesh_occlusion;

/// The names a run's program reads but never defines: the effective palette's
/// properties it reads, one swatch array each, and the computed bindings. The
/// types and the property values hold for every geometry. The computed values
/// and the groupings follow each geometry.
pub struct MeshEnvironment {
    pub types: TypeEnvironment,

    properties: HashMap<String, Value>,

    computed_bindings: Vec<ComputedBinding>,
}

impl MeshEnvironment {
    /// Binds each property `swatches` read through that `free_names` holds. A
    /// computed binding gets only its type here because its value waits for a
    /// geometry. Errors on a computed binding shadowing a property or bound
    /// twice.
    pub(crate) fn bind(
        swatches: &Swatches<'_>,
        computed_bindings: &[ComputedBinding],
        free_names: &BTreeSet<String>,
    ) -> Result<Self> {
        let mut types = HashMap::new();
        let mut properties = HashMap::new();

        let effective = swatches.effective();

        for property_id in IdRange::from_len(effective.property_count()) {
            let property = effective
                .property(property_id)
                .expect("property ids below the count resolve");

            if !free_names.contains(property.name()) {
                continue;
            }

            let swatch_value_ids: Vec<_> = IdRange::from_len(swatches.count())
                .map(|swatch_id| swatches.value_id(swatch_id, property_id))
                .collect();

            if let Some(value) =
                property_value(property.name(), property.value_pool(), &swatch_value_ids)?
            {
                types.insert(property.name().to_owned(), value.to_type());
                properties.insert(property.name().to_owned(), value);
            }
        }

        for binding in computed_bindings {
            let element = MeshElement::ComputedBinding {
                name: binding.name.clone(),
            };

            if effective.property_id_by_name(&binding.name).is_some() {
                return Err(Error::mesh_record(
                    element,
                    "shadows the palette property of the same name",
                ));
            }

            if types.contains_key(&binding.name) {
                return Err(Error::mesh_record(element, "is bound twice"));
            }

            types.insert(binding.name.clone(), computed_type(binding.computation));
        }

        Ok(MeshEnvironment {
            types: TypeEnvironment { types },
            properties,
            computed_bindings: computed_bindings.to_vec(),
        })
    }

    /// The value environment over `geometry`.
    pub(crate) fn values(
        &self,
        object: &VoxObject,
        swatches: &Swatches<'_>,
        geometry: &MeshGeometry,
    ) -> ValueEnvironment {
        let entries = |domain: Domain| match domain {
            Domain::Corner => geometry.quad_count() * 4,
            Domain::Face => geometry.quad_count(),
            Domain::Plain => unreachable!("a computed index runs over an array domain"),
            Domain::Swatch => swatches.count(),
            Domain::Voxel => swatches.voxel_swatch_ids().len(),
        };

        let mut values = self.properties.clone();

        for binding in &self.computed_bindings {
            let value = match binding.computation {
                Computation::Index(domain) => {
                    let domain = Domain::from(domain);
                    compute_index(domain, entries(domain))
                }

                Computation::Occlusion => {
                    compute_occlusion(&SwatchGrid::new(object, swatches), geometry)
                }

                Computation::VoxelPosition => compute_voxel_position(object),
            };

            values.insert(binding.name.clone(), value);
        }

        ValueEnvironment {
            values,
            groupings: groupings_of(swatches, geometry),
        }
    }
}

/// The type `computation` binds over any geometry.
fn computed_type(computation: Computation) -> Type {
    let (domain, dimension, scalar) = match computation {
        Computation::Index(domain) => (Domain::from(domain), Dimension::Vec1, Scalar::U32),
        Computation::Occlusion => (Domain::Corner, Dimension::Vec1, Scalar::F64),
        Computation::VoxelPosition => (Domain::Voxel, Dimension::Vec3, Scalar::U32),
    };

    Type {
        domain,
        dimension,
        scalar,
    }
}

/// Each entry's index in `domain`, a `u32` vec1 array of `count` entries
/// counting up from `0`.
fn compute_index(domain: Domain, count: usize) -> Value {
    Value::new(
        domain,
        Dimension::Vec1,
        Components::U32((0..count).map(|index| index as u32).collect()),
    )
    .expect("one component per entry fills a vec1 array")
}

/// Each live voxel's grid position from the minimum corner of the live
/// extent, a voxel `u32` vec3 array in raster order.
fn compute_voxel_position(object: &VoxObject) -> Value {
    let origin = object
        .live_extent()
        .map(|(minimum, _)| minimum.to_array())
        .unwrap_or([0; 3]);

    let mut components = Vec::with_capacity(object.live_count() * 3);

    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid")
            .to_array();

        components.extend((0..3).map(|axis| position[axis] - origin[axis]));
    }

    Value::new(Domain::Voxel, Dimension::Vec3, Components::U32(components))
        .expect("three components per voxel fill a vec3 array")
}

/// Each face corner's [`mesh_occlusion`] as a corner `f64` vec1 array.
fn compute_occlusion(grid: &SwatchGrid<'_>, geometry: &MeshGeometry) -> Value {
    Value::new(
        Domain::Corner,
        Dimension::Vec1,
        Components::F64(mesh_occlusion(grid, geometry)),
    )
    .expect("one component per corner fills a vec1 array")
}

/// The groupings over `geometry`. Each face piece maps to its voxel's entry.
fn groupings_of(swatches: &Swatches<'_>, geometry: &MeshGeometry) -> Groupings {
    Groupings {
        swatch_count: swatches.count(),
        voxel_swatches: swatches.voxel_swatch_ids().clone(),
        face_voxels: geometry
            .face_cells
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
    use crate::{
        operations::object::{
            ArrayDomain, Computation, Method, SwatchGrid, Swatches,
            object_mesh::program::mesh_environment::{
                compute_index, compute_occlusion, compute_voxel_position, computed_type,
                groupings_of,
            },
        },
        test_utilities::live_object,
    };
    use vox_value_language::{Components, Domain};
    use voxcore::VoxMain;
    use voxsurface::mesh_grid;

    #[test]
    fn the_index_counts_the_entries_up() {
        let value = compute_index(Domain::Face, 3);

        assert_eq!(value.domain(), Domain::Face);
        assert_eq!(value.components(), &Components::U32(vec![0, 1, 2]));
    }

    #[test]
    fn positions_start_at_the_live_extent_in_raster_order() {
        let object = live_object([4, 4, 4], &[[3, 2, 1], [1, 2, 1], [1, 3, 2]]);

        let value = compute_voxel_position(&object);

        assert_eq!(value.domain(), Domain::Voxel);
        assert_eq!(
            value.components(),
            &Components::U32(vec![0, 0, 0, 0, 1, 1, 2, 0, 0])
        );
    }

    #[test]
    fn the_occlusion_lands_one_corner_entry_per_vertex() {
        let main: VoxMain = VoxMain::default();
        let object = live_object([3, 3, 3], &[[1, 1, 1]]);
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let geometry = mesh_grid(&object, Method::Culled);

        let value = compute_occlusion(&SwatchGrid::new(&object, &swatches), &geometry);

        assert_eq!(value.domain(), Domain::Corner);
        assert_eq!(value.entries(), 24);
        assert_eq!(value.components(), &Components::F64(vec![1.0; 24]));
    }

    #[test]
    fn each_computation_binds_its_computed_type() {
        let main: VoxMain = VoxMain::default();
        let object = live_object([3, 3, 3], &[[1, 1, 1]]);
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let geometry = mesh_grid(&object, Method::Culled);

        let cases = [
            (
                Computation::Index(ArrayDomain::Face),
                compute_index(Domain::Face, 6),
            ),
            (
                Computation::Occlusion,
                compute_occlusion(&SwatchGrid::new(&object, &swatches), &geometry),
            ),
            (Computation::VoxelPosition, compute_voxel_position(&object)),
        ];

        for (computation, value) in cases {
            assert_eq!(
                computed_type(computation),
                value.to_type(),
                "{computation:?}"
            );
        }
    }

    #[test]
    fn a_merged_face_lists_every_voxel_it_covers() {
        let main: VoxMain = VoxMain::default();
        let object = live_object([2, 1, 1], &[[0, 0, 0], [1, 0, 0]]);
        let swatches = Swatches::resolve(&main, &object).unwrap();
        let geometry = mesh_grid(&object, Method::Greedy);

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
