use crate::{
    Error, Result,
    operations::object::{Computation, ComputedBinding, MeshElement, MeshGeometry, Swatches},
};
use branded_id::{U32Id, UsizeId};
use std::{
    collections::{BTreeSet, HashMap},
    slice,
};
use vox_value_language::{
    Components, Dimension, Domain, Groupings, Scalar, Type, TypeEnvironment, Value,
    ValueEnvironment,
};
use voxcore::{VoxObject, VoxValuePoolKind, VoxValuePoolValueRef};
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

        for index in 0..effective.property_count() {
            let property_id = UsizeId::from_usize(index);

            let property = effective
                .property(property_id)
                .expect("property ids below the count resolve");

            if !free_names.contains(property.name()) {
                continue;
            }

            let swatch_values: Vec<_> = (0..swatches.count())
                .map(|swatch| swatches.value(U32Id::from_u32(swatch as u32), property_id))
                .collect();

            if let Some(value) = property_value(
                property.name(),
                property.value_pool().kind(),
                &swatch_values,
            )? {
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

                Computation::Occlusion => compute_occlusion(object, geometry),

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

/// The swatch array of the property `name`, whose pool has `kind`, from its
/// per-swatch `values`, or `None` for a json property, which the language
/// has no type for. An int reads as `u32` and errors outside its range.
fn property_value(
    name: &str,
    kind: &VoxValuePoolKind,
    values: &[VoxValuePoolValueRef<'_>],
) -> Result<Option<Value>> {
    let (dimension, components) = match kind {
        VoxValuePoolKind::Bool(_) => (
            Dimension::Vec1,
            Components::Bool(
                values
                    .iter()
                    .map(|value| match value {
                        VoxValuePoolValueRef::Bool(value) => *value,
                        _ => unreachable!("a pool's values share its kind"),
                    })
                    .collect(),
            ),
        ),

        VoxValuePoolKind::Float(_) => (
            Dimension::Vec1,
            f64s(values, |value| match value {
                VoxValuePoolValueRef::Float(value) => slice::from_ref(value),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Int(_) => (
            Dimension::Vec1,
            u32s(name, values, 1, |value| match value {
                VoxValuePoolValueRef::Int(value) => slice::from_ref(value),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Json(_) => return Ok(None),

        VoxValuePoolKind::String(_) => (
            Dimension::Vec1,
            Components::String(
                values
                    .iter()
                    .map(|value| match value {
                        VoxValuePoolValueRef::String(value) => (*value).to_owned(),
                        _ => unreachable!("a pool's values share its kind"),
                    })
                    .collect(),
            ),
        ),

        VoxValuePoolKind::Vec2Float(_) => (
            Dimension::Vec2,
            f64s(values, |value| match value {
                VoxValuePoolValueRef::Vec2Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec2Int(_) => (
            Dimension::Vec2,
            u32s(name, values, 2, |value| match value {
                VoxValuePoolValueRef::Vec2Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Vec3Float(_) => (
            Dimension::Vec3,
            f64s(values, |value| match value {
                VoxValuePoolValueRef::Vec3Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec3Int(_) => (
            Dimension::Vec3,
            u32s(name, values, 3, |value| match value {
                VoxValuePoolValueRef::Vec3Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),

        VoxValuePoolKind::Vec4Float(_) => (
            Dimension::Vec4,
            f64s(values, |value| match value {
                VoxValuePoolValueRef::Vec4Float(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            }),
        ),

        VoxValuePoolKind::Vec4Int(_) => (
            Dimension::Vec4,
            u32s(name, values, 4, |value| match value {
                VoxValuePoolValueRef::Vec4Int(components) => components.as_slice(),
                _ => unreachable!("a pool's values share its kind"),
            })?,
        ),
    };

    let value = Value::new(Domain::Swatch, dimension, components)
        .expect("every swatch contributes one entry of the pool's width");

    Ok(Some(value))
}

/// The float components of `values` flattened, each read through `pick`.
fn f64s<'a>(
    values: &[VoxValuePoolValueRef<'a>],
    pick: for<'b> fn(&'b VoxValuePoolValueRef<'a>) -> &'b [f64],
) -> Components {
    Components::F64(
        values
            .iter()
            .flat_map(|value| pick(value).iter().copied())
            .collect(),
    )
}

/// The int components of `values` flattened as `u32`, each read through
/// `pick`, erroring on one outside the range and reporting its swatch.
fn u32s<'a>(
    name: &str,
    values: &[VoxValuePoolValueRef<'a>],
    width: usize,
    pick: for<'b> fn(&'b VoxValuePoolValueRef<'a>) -> &'b [i64],
) -> Result<Components> {
    let mut components = Vec::with_capacity(values.len() * width);

    for (swatch, value) in (0..).zip(values) {
        for &component in pick(value) {
            let component = u32::try_from(component).map_err(|_| {
                Error::invalid(format!(
                    "the property `{name}` holds {component} at swatch {swatch}, and the value \
                     language reads an int as u32"
                ))
            })?;

            components.push(component);
        }
    }

    Ok(Components::U32(components))
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
fn compute_occlusion(object: &VoxObject, geometry: &MeshGeometry) -> Value {
    Value::new(
        Domain::Corner,
        Dimension::Vec1,
        Components::F64(mesh_occlusion(object, geometry)),
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
            ArrayDomain, Computation, Method, Swatches,
            object_mesh::program::mesh_environment::{
                compute_index, compute_occlusion, compute_voxel_position, computed_type,
                groupings_of, property_value,
            },
        },
        test_utilities::live_object,
    };
    use vox_value_language::{Components, Dimension, Domain, Scalar};
    use voxcore::{VoxMain, VoxValue, VoxValuePool, VoxValuePoolValueRef};
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
        let object = live_object([3, 3, 3], &[[1, 1, 1]]);
        let geometry = mesh_grid(&object, Method::Culled);

        let value = compute_occlusion(&object, &geometry);

        assert_eq!(value.domain(), Domain::Corner);
        assert_eq!(value.entries(), 24);
        assert_eq!(value.components(), &Components::F64(vec![1.0; 24]));
    }

    #[test]
    fn each_computation_binds_its_computed_type() {
        let object = live_object([3, 3, 3], &[[1, 1, 1]]);
        let geometry = mesh_grid(&object, Method::Culled);

        let cases = [
            (
                Computation::Index(ArrayDomain::Face),
                compute_index(Domain::Face, 6),
            ),
            (
                Computation::Occlusion,
                compute_occlusion(&object, &geometry),
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

    #[test]
    fn each_pool_kind_binds_its_language_type() {
        let cases: [(VoxValuePool, Dimension, Scalar, usize); 6] = [
            (
                VoxValuePool::boolean(vec![true]),
                Dimension::Vec1,
                Scalar::Bool,
                1,
            ),
            (
                VoxValuePool::float(vec![0.5]).unwrap(),
                Dimension::Vec1,
                Scalar::F64,
                1,
            ),
            (
                VoxValuePool::int(vec![7]).unwrap(),
                Dimension::Vec1,
                Scalar::U32,
                1,
            ),
            (
                VoxValuePool::string(vec!["glass".to_owned()]),
                Dimension::Vec1,
                Scalar::String,
                1,
            ),
            (
                VoxValuePool::vec_3_int(vec![[1, 2, 3]]).unwrap(),
                Dimension::Vec3,
                Scalar::U32,
                3,
            ),
            (
                VoxValuePool::vec_4_float(vec![[0.0, 0.5, 1.0, 1.0]]).unwrap(),
                Dimension::Vec4,
                Scalar::F64,
                4,
            ),
        ];

        for (pool, dimension, scalar, components) in cases {
            let values: Vec<VoxValuePoolValueRef> =
                pool.iter_values().map(|(_, value)| value).collect();

            let value = property_value("p", pool.kind(), &values).unwrap().unwrap();

            assert_eq!(value.domain(), Domain::Swatch, "{dimension} {scalar}");
            assert_eq!(value.dimension(), dimension, "{dimension} {scalar}");
            assert_eq!(value.scalar(), scalar, "{dimension} {scalar}");
            assert_eq!(value.components().len(), components, "{dimension} {scalar}");
        }
    }

    #[test]
    fn swatches_flatten_in_order() {
        let pool = VoxValuePool::vec_2_float(vec![[1.0, 2.0], [3.0, 4.0]]).unwrap();
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        let value = property_value("p", pool.kind(), &values).unwrap().unwrap();

        assert_eq!(value.entries(), 2);
        assert_eq!(
            value.components(),
            &Components::F64(vec![1.0, 2.0, 3.0, 4.0])
        );
    }

    #[test]
    fn an_int_outside_u32_errors_at_its_swatch() {
        let pool = VoxValuePool::int(vec![1, -1]).unwrap();
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        let error = property_value("tag", pool.kind(), &values)
            .unwrap_err()
            .to_string();

        assert!(error.contains("`tag` holds -1 at swatch 1"), "{error}");
    }

    #[test]
    fn a_json_property_binds_nothing() {
        let pool = VoxValuePool::json(vec![VoxValue::Null]);
        let values: Vec<_> = pool.iter_values().map(|(_, value)| value).collect();

        assert_eq!(property_value("meta", pool.kind(), &values).unwrap(), None);
    }
}
