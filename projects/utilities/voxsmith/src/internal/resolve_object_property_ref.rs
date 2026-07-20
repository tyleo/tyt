use crate::ObjectPropertyRef;
use voxcore::{VoxMain, VoxObject, VoxPropertyId};

/// The winning supplier of property `name` for `object`, or `None` when no
/// layer supplies it. The scan passes over a missing palette and an array
/// property in an unsampled layer.
pub fn resolve_object_property_ref(
    state: &VoxMain,
    object: &VoxObject,
    name: &str,
) -> Option<ObjectPropertyRef> {
    let layers: Vec<_> = object.iter_layers().collect();

    layers.into_iter().rev().find_map(|(layer, palette_id)| {
        let palette = state.palette(palette_id)?;

        match palette.property_by_name(name)? {
            VoxPropertyId::Scalar(property) => Some(ObjectPropertyRef::Scalar {
                palette: palette_id,
                property,
            }),
            VoxPropertyId::Array(property) if palette.material_count() > 0 => {
                Some(ObjectPropertyRef::Array {
                    layer,
                    palette: palette_id,
                    property,
                })
            }
            VoxPropertyId::Array(_) => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, ObjectPropertyRef, resolve_object_property_ref};
    use branded_id::{IdVec, U32Id};
    use ty_math::TyVector3U32;
    use voxcore::{BVoxPalette, BVoxPoolValue, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The branded value id `index`.
    fn value(index: u32) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index)
    }

    /// A one-material palette whose `baseColorFactor` array property draws
    /// from a fresh one-color pool.
    fn array_palette(state: &mut VoxMain) -> U32Id<BVoxPalette> {
        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[1.0, 0.0, 0.0, 1.0]]),
        });

        let mut palette = VoxPalette::default();
        palette.add_array_property(BASE_COLOR_FACTOR.to_owned(), pool);
        palette.add_material(vec![value(0)]).unwrap();

        state.add_palette(palette)
    }

    /// A one-voxel object layering the given palettes in order.
    fn object_over(palettes: &[U32Id<BVoxPalette>]) -> VoxObject {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for &palette in palettes {
            object.add_layer(palette, U32Id::from_u32(0));
        }
        object
    }

    #[test]
    fn the_last_supplying_layer_wins() {
        let mut state = VoxMain::default();
        let first = array_palette(&mut state);
        let second = array_palette(&mut state);
        let object = object_over(&[first, second]);

        let winner = resolve_object_property_ref(&state, &object, BASE_COLOR_FACTOR).unwrap();
        let layers: Vec<_> = object.iter_layers().collect();
        assert_eq!(
            winner,
            ObjectPropertyRef::Array {
                layer: layers[1].0,
                palette: second,
                property: state
                    .palette(second)
                    .unwrap()
                    .array_property_by_name(BASE_COLOR_FACTOR)
                    .unwrap(),
            }
        );
    }

    #[test]
    fn a_scalar_layer_listed_after_an_array_layer_wins() {
        let mut state = VoxMain::default();
        let array = array_palette(&mut state);

        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[0.0, 0.0, 1.0, 1.0]]),
        });
        let mut palette = VoxPalette::default();
        let property = palette.add_scalar_property(BASE_COLOR_FACTOR.to_owned(), pool, value(0));
        let scalar = state.add_palette(palette);

        let object = object_over(&[array, scalar]);

        assert_eq!(
            resolve_object_property_ref(&state, &object, BASE_COLOR_FACTOR),
            Some(ObjectPropertyRef::Scalar {
                palette: scalar,
                property,
            })
        );
    }

    #[test]
    fn an_unsampled_array_layer_is_passed_over() {
        let mut state = VoxMain::default();
        let sampled = array_palette(&mut state);

        // The trailing palette carries the name as an array property but has
        // no materials, so its layer is unsampled and the scan passes it.
        let pool = state.add_value_pool(VoxValuePool::Srgba {
            values: IdVec::from_vec(vec![[0.0, 1.0, 0.0, 1.0]]),
        });
        let mut palette = VoxPalette::default();
        palette.add_array_property(BASE_COLOR_FACTOR.to_owned(), pool);
        let unsampled = state.add_palette(palette);

        let object = object_over(&[sampled, unsampled]);

        let winner = resolve_object_property_ref(&state, &object, BASE_COLOR_FACTOR).unwrap();
        let layers: Vec<_> = object.iter_layers().collect();
        assert!(matches!(
            winner,
            ObjectPropertyRef::Array { layer, palette, .. }
                if layer == layers[0].0 && palette == sampled
        ));
    }

    #[test]
    fn none_when_no_layer_supplies_the_name() {
        let mut state = VoxMain::default();
        let palette = array_palette(&mut state);
        let object = object_over(&[palette]);

        assert_eq!(
            resolve_object_property_ref(&state, &object, "emissiveStrength"),
            None
        );
    }
}
