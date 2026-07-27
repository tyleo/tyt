use crate::ObjectPropertyRef;
use voxcore::{VoxMain, VoxObject};

/// The winning supplier of property `name` for `object`, or `None` when no
/// layer supplies it. The scan passes over a missing palette.
pub fn resolve_object_property_ref(
    state: &VoxMain,
    object: &VoxObject,
    name: &str,
) -> Option<ObjectPropertyRef> {
    let layers: Vec<_> = object.iter_layers().collect();

    layers.into_iter().rev().find_map(|(layer, palette_id)| {
        let palette = state.palette(palette_id)?;
        let property = palette.property_by_name(name)?;
        Some(ObjectPropertyRef {
            layer,
            palette: palette_id,
            property,
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, ObjectPropertyRef, resolve_object_property_ref};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxPalette, BVoxPoolValue, VoxMain, VoxObject, VoxPalette, VoxValuePool};

    /// The branded value id `index`.
    fn value(index: u32) -> U32Id<BVoxPoolValue> {
        U32Id::from_u32(index)
    }

    /// A one-material palette whose `baseColorFactor` property draws
    /// from a fresh one-color pool.
    fn color_palette(state: &mut VoxMain) -> U32Id<BVoxPalette> {
        let pool = state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]));

        let mut palette = VoxPalette::default();
        palette
            .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
            .unwrap();
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
        let first = color_palette(&mut state);
        let second = color_palette(&mut state);
        let object = object_over(&[first, second]);

        let winner = resolve_object_property_ref(&state, &object, BASE_COLOR_FACTOR).unwrap();
        let layers: Vec<_> = object.iter_layers().collect();
        assert_eq!(
            winner,
            ObjectPropertyRef {
                layer: layers[1].0,
                palette: second,
                property: state
                    .palette(second)
                    .unwrap()
                    .property_by_name(BASE_COLOR_FACTOR)
                    .unwrap(),
            }
        );
    }

    #[test]
    fn a_layer_without_the_property_is_passed_over() {
        let mut state = VoxMain::default();
        let supplying = color_palette(&mut state);

        // The trailing palette does not carry the name, so the scan passes
        // its layer.
        let pool = state.add_value_pool(VoxValuePool::srgba(vec![[0.0, 1.0, 0.0, 1.0]]));
        let mut palette = VoxPalette::default();
        palette
            .add_property("emissiveFactor".to_owned(), pool)
            .unwrap();
        palette.add_material(vec![value(0)]).unwrap();
        let plain = state.add_palette(palette);

        let object = object_over(&[supplying, plain]);

        let winner = resolve_object_property_ref(&state, &object, BASE_COLOR_FACTOR).unwrap();
        let layers: Vec<_> = object.iter_layers().collect();
        assert!(winner.layer == layers[0].0 && winner.palette == supplying);
    }

    #[test]
    fn none_when_no_layer_supplies_the_name() {
        let mut state = VoxMain::default();
        let palette = color_palette(&mut state);
        let object = object_over(&[palette]);

        assert_eq!(
            resolve_object_property_ref(&state, &object, "emissiveStrength"),
            None
        );
    }
}
