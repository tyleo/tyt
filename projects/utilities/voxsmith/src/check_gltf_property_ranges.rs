use crate::{COLOR_RANGE, GltfPropertyKind, GltfRange, Result, scalar_range};
use voxcore::{VoxMain, VoxValuePoolValueRef};

/// Checks every palette property named by the glTF vocabulary against that
/// name's glTF schema range, erroring on the first value a material draws
/// outside it, `ior`'s union of exactly `0` and `[1, inf)` included.
///
/// A range is a fact about the property name, not the format, so a value pool
/// loads unchecked and the glTF boundaries run this one function instead of
/// growing their own:
///
/// 1. the export before writing, so nothing out of range reaches a mesh file
/// 2. the import on what it read, so a bad source errors at entry
///
/// A name outside the vocabulary has no checkable range and passes, as does a
/// value of a shape the name does not read. The boundary that reads the value
/// errors on its shape.
pub fn check_gltf_property_ranges(state: &VoxMain) -> Result<()> {
    for (palette_id, palette) in state.iter_palettes() {
        for (property_id, property) in palette.iter_properties() {
            let name = &property.name;
            let Some(kind) = GltfPropertyKind::of(name) else {
                continue;
            };
            let range = match kind {
                GltfPropertyKind::ColorRgb | GltfPropertyKind::ColorRgba => COLOR_RANGE,
                GltfPropertyKind::Scalar => {
                    scalar_range(name).expect("every scalar vocabulary property has a range")
                }
            };

            for material_id in palette.iter_materials() {
                let Some(value) = state
                    .material_value(palette_id, material_id, property_id)
                    .and_then(|(value_pool, value_id)| value_pool.value(value_id))
                else {
                    continue;
                };

                match kind {
                    GltfPropertyKind::ColorRgb | GltfPropertyKind::ColorRgba => match value {
                        VoxValuePoolValueRef::Vec3Float(components) => {
                            check_components(name, components, range)?
                        }
                        VoxValuePoolValueRef::Vec4Float(components) => {
                            check_components(name, components, range)?
                        }
                        _ => {}
                    },
                    GltfPropertyKind::Scalar => match value {
                        VoxValuePoolValueRef::Float(number) => range.check(name, number)?,
                        VoxValuePoolValueRef::Int(number) => range.check(name, number as f64)?,
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}

/// Errors unless every component of a vocabulary color lies in `range`.
fn check_components(name: &str, components: &[f64], range: GltfRange) -> Result<()> {
    for &component in components {
        range.check(name, component)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, check_gltf_property_ranges};
    use branded_id::U32Id;
    use voxcore::{VoxMain, VoxPalette, VoxValuePool};

    /// A state whose one palette binds `name` to a one-material value pool.
    fn state_with(name: &str, value_pool: VoxValuePool) -> VoxMain {
        let mut state = VoxMain::default();
        let value_pool_id = state.retain_value_pool(value_pool);

        let mut palette = VoxPalette::default();
        palette
            .retain_property(name.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        state.retain_palette(palette).unwrap();

        state
    }

    #[test]
    fn passes_in_range_values_and_unranged_names() {
        let state = state_with(METALLIC, VoxValuePool::float(vec![1.0]).unwrap());
        assert!(check_gltf_property_ranges(&state).is_ok());

        // `emissiveStrength` is unbounded above.
        let state = state_with(EMISSIVE_STRENGTH, VoxValuePool::float(vec![7.0]).unwrap());
        assert!(check_gltf_property_ranges(&state).is_ok());

        // A custom name has no checkable range.
        let state = state_with("subsurface", VoxValuePool::float(vec![7.0]).unwrap());
        assert!(check_gltf_property_ranges(&state).is_ok());
    }

    #[test]
    fn errors_on_a_scalar_outside_its_range() {
        let state = state_with(METALLIC, VoxValuePool::float(vec![1.5]).unwrap());
        let message = check_gltf_property_ranges(&state).unwrap_err().to_string();
        assert!(message.contains(METALLIC), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn spells_the_ior_union_exactly() {
        // Zero means "does not refract" and passes. A value between the
        // union's parts rejects.
        let state = state_with(IOR, VoxValuePool::float(vec![0.0, 1.5]).unwrap());
        assert!(check_gltf_property_ranges(&state).is_ok());

        let state = state_with(IOR, VoxValuePool::float(vec![0.5]).unwrap());
        let message = check_gltf_property_ranges(&state).unwrap_err().to_string();
        assert!(message.contains(IOR), "{message}");
    }

    #[test]
    fn errors_on_a_color_component_outside_zero_to_one() {
        let state = state_with(
            BASE_COLOR,
            VoxValuePool::vec_4_float(vec![[1.5, 0.0, 0.0, 1.0]]).unwrap(),
        );
        let message = check_gltf_property_ranges(&state).unwrap_err().to_string();
        assert!(message.contains(BASE_COLOR), "{message}");
    }

    #[test]
    fn skips_a_value_no_material_draws() {
        // The second value is out of range, and no material draws it: only
        // drawn values reach a glTF factor, so it passes.
        let state = state_with(METALLIC, VoxValuePool::float(vec![1.0, 7.0]).unwrap());
        assert!(check_gltf_property_ranges(&state).is_ok());
    }
}
