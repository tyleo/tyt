use crate::{
    Error, Result,
    operations::mesh::{
        Computation, ComputedBinding, MeshElement, MeshGeometry, Swatches, compute_index,
        compute_occlusion, compute_voxel_position, groupings_of, property_value,
    },
};
use branded_id::{U32Id, UsizeId};
use std::collections::HashMap;
use vox_value_language::{Domain, TypeEnvironment, ValueEnvironment};
use voxcore::VoxObject;

/// The names a run's program reads but never defines: the effective palette's
/// properties, one swatch array each, and the computed bindings.
pub(crate) struct MeshEnvironment {
    pub types: TypeEnvironment,
    pub values: ValueEnvironment,
}

impl MeshEnvironment {
    /// Binds the properties `swatches` read through and `computed_bindings`
    /// over `object` and `geometry`. Errors on a computed binding shadowing
    /// a property or bound twice.
    pub(crate) fn bind(
        object: &VoxObject,
        swatches: &Swatches<'_>,
        geometry: &MeshGeometry,
        computed_bindings: &[ComputedBinding],
    ) -> Result<Self> {
        let mut types = HashMap::new();
        let mut values = HashMap::new();

        let effective = swatches.effective();

        for index in 0..effective.property_count() {
            let property_id = UsizeId::from_usize(index);

            let property = effective
                .property(property_id)
                .expect("property ids below the count resolve");

            let swatch_values: Vec<_> = (0..swatches.count())
                .map(|swatch| swatches.value(U32Id::from_u32(swatch as u32), property_id))
                .collect();

            if let Some(value) = property_value(
                property.name(),
                property.value_pool().kind(),
                &swatch_values,
            )? {
                types.insert(property.name().to_owned(), value.to_type());
                values.insert(property.name().to_owned(), value);
            }
        }

        let entries = |domain: Domain| match domain {
            Domain::Corner => geometry.quad_count() * 4,
            Domain::Face => geometry.quad_count(),
            Domain::Plain => unreachable!("a computed index runs over an array domain"),
            Domain::Swatch => swatches.count(),
            Domain::Voxel => swatches.voxel_swatch_ids().len(),
        };

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

            if values.contains_key(&binding.name) {
                return Err(Error::mesh_record(element, "is bound twice"));
            }

            let value = match binding.computation {
                Computation::Index(domain) => {
                    let domain = Domain::from(domain);
                    compute_index(domain, entries(domain))
                }

                Computation::Occlusion => compute_occlusion(object, geometry),

                Computation::VoxelPosition => compute_voxel_position(object),
            };

            types.insert(binding.name.clone(), value.to_type());
            values.insert(binding.name.clone(), value);
        }

        Ok(MeshEnvironment {
            types: TypeEnvironment { types },
            values: ValueEnvironment {
                values,
                groupings: groupings_of(swatches, geometry),
            },
        })
    }
}
