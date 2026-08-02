use crate::{BASE_COLOR, EMISSIVE_COLOR, Error, GltfRange, Result, scalar_range};
use voxcore::{VoxMain, VoxValuePoolValueRef};

/// The `[0, 1]` every component of a vocabulary color lies in, per the glTF
/// schema of `baseColorFactor` and `emissiveFactor`.
const COLOR_RANGE: GltfRange = GltfRange {
    min: 0.0,
    max: Some(1.0),
    admits_zero: false,
};

/// Checks every palette property named by the glTF vocabulary against that
/// name's glTF schema range, erroring on the first value a material draws
/// outside it, `ior`'s union of exactly `0` and `[1, inf)` included.
///
/// A range is a fact about the property name, not the format, so a value
/// pool loads unchecked and this one function is what the glTF boundaries
/// run: the export before writing, so nothing out of range reaches a mesh
/// file, and the import on what it read, so a bad source errors at entry. A
/// name outside the vocabulary has no checkable range and passes, as does a
/// value of a shape the name does not read; the boundary that reads it
/// errors on the shape itself.
pub fn check_gltf_attribute_ranges(state: &VoxMain) -> Result<()> {
    for (palette_id, palette) in state.iter_palettes() {
        for (property_id, property) in palette.iter_properties() {
            let scalar = scalar_range(&property.name);
            let color = matches!(property.name.as_str(), BASE_COLOR | EMISSIVE_COLOR);
            if scalar.is_none() && !color {
                continue;
            }

            for material_id in palette.iter_materials() {
                let Some(value) = state
                    .material_value(palette_id, material_id, property_id)
                    .and_then(|(value_pool, value_id)| value_pool.value(value_id))
                else {
                    continue;
                };

                match (value, scalar) {
                    (VoxValuePoolValueRef::Float(number), Some(range)) => {
                        check(&property.name, number, range)?
                    }
                    (VoxValuePoolValueRef::Int(number), Some(range)) => {
                        check(&property.name, number as f64, range)?
                    }
                    (VoxValuePoolValueRef::Vec3Float(components), None) if color => {
                        check_components(&property.name, components)?
                    }
                    (VoxValuePoolValueRef::Vec4Float(components), None) if color => {
                        check_components(&property.name, components)?
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Errors unless `value` lies in `range`.
fn check(name: &str, value: f64, range: GltfRange) -> Result<()> {
    if range.contains(value) {
        Ok(())
    } else {
        Err(Error::invalid(format!(
            "`{name}` is {value}, outside the glTF range {range}"
        )))
    }
}

/// Errors unless every component of a vocabulary color lies in `[0, 1]`.
fn check_components(name: &str, components: &[f64]) -> Result<()> {
    for &component in components {
        if !COLOR_RANGE.contains(component) {
            return Err(Error::invalid(format!(
                "`{name}` component is {component}, outside the glTF range {COLOR_RANGE}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, check_gltf_attribute_ranges};
    use branded_id::U32Id;
    use voxcore::{VoxMain, VoxPalette, VoxValuePool};

    /// A state whose one palette binds `name` to a one-material value pool.
    fn state_with(name: &str, value_pool: VoxValuePool) -> VoxMain {
        let mut state = VoxMain::default();
        let value_pool_id = state.add_value_pool(value_pool);

        let mut palette = VoxPalette::default();
        palette
            .add_property(name.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        state.add_palette(palette).unwrap();

        state
    }

    #[test]
    fn passes_in_range_values_and_unranged_names() {
        let state = state_with(METALLIC, VoxValuePool::float(vec![1.0]).unwrap());
        assert!(check_gltf_attribute_ranges(&state).is_ok());

        // `emissiveStrength` is unbounded above.
        let state = state_with(EMISSIVE_STRENGTH, VoxValuePool::float(vec![7.0]).unwrap());
        assert!(check_gltf_attribute_ranges(&state).is_ok());

        // A custom name has no checkable range.
        let state = state_with("subsurface", VoxValuePool::float(vec![7.0]).unwrap());
        assert!(check_gltf_attribute_ranges(&state).is_ok());
    }

    #[test]
    fn errors_on_a_scalar_outside_its_range() {
        let state = state_with(METALLIC, VoxValuePool::float(vec![1.5]).unwrap());
        let message = check_gltf_attribute_ranges(&state).unwrap_err().to_string();
        assert!(message.contains(METALLIC), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn spells_the_ior_union_exactly() {
        // Zero means "does not refract" and passes; between the union's parts
        // rejects.
        let state = state_with(IOR, VoxValuePool::float(vec![0.0, 1.5]).unwrap());
        assert!(check_gltf_attribute_ranges(&state).is_ok());

        let state = state_with(IOR, VoxValuePool::float(vec![0.5]).unwrap());
        let message = check_gltf_attribute_ranges(&state).unwrap_err().to_string();
        assert!(message.contains(IOR), "{message}");
    }

    #[test]
    fn errors_on_a_color_component_outside_zero_to_one() {
        let state = state_with(
            BASE_COLOR,
            VoxValuePool::vec_4_float(vec![[1.5, 0.0, 0.0, 1.0]]).unwrap(),
        );
        let message = check_gltf_attribute_ranges(&state).unwrap_err().to_string();
        assert!(message.contains(BASE_COLOR), "{message}");
    }

    #[test]
    fn skips_a_value_no_material_draws() {
        // The second value is out of range, and no material draws it: only
        // drawn values reach a glTF factor, so it passes.
        let state = state_with(METALLIC, VoxValuePool::float(vec![1.0, 7.0]).unwrap());
        assert!(check_gltf_attribute_ranges(&state).is_ok());
    }
}
