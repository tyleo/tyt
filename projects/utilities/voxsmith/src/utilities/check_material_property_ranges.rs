use crate::{Result, utilities::check_material_range};
use meshdoc::material::{COLOR_RANGE, MaterialPropertyKind, MaterialRange, scalar_range};
use voxcore::{VoxExt, VoxMain, VoxValuePoolValueRef};

/// Checks every palette property in the material vocabulary against its
/// range, erroring on the first value a material draws outside it.
///
/// A value pool loads unchecked because a range belongs to the property
/// name. Both mesh boundaries run this one check:
///
/// 1. the mesh export before building the document
/// 2. the voxelize import on what it read
///
/// A name outside the vocabulary has no range and passes, as does a value of
/// a shape the name does not read. The boundary that reads the value errors
/// on its shape.
pub fn check_material_property_ranges<T: VoxExt>(main: &VoxMain<T>) -> Result<()> {
    for (palette_id, palette) in main.iter_palettes() {
        for (property_id, property) in palette.iter_properties() {
            let name = &property.name;
            let Some(kind) = MaterialPropertyKind::of(name) else {
                continue;
            };
            let range = match kind {
                MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba => COLOR_RANGE,
                MaterialPropertyKind::Scalar => {
                    scalar_range(name).expect("every scalar vocabulary property has a range")
                }
            };

            for material_id in palette.iter_materials() {
                let Some(value) = main
                    .material_value(palette_id, material_id, property_id)
                    .and_then(|(value_pool, value_id)| value_pool.value(value_id))
                else {
                    continue;
                };

                match kind {
                    MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba => match value
                    {
                        VoxValuePoolValueRef::Vec3Float(components) => {
                            check_components(name, components, range)?
                        }
                        VoxValuePoolValueRef::Vec4Float(components) => {
                            check_components(name, components, range)?
                        }
                        _ => {}
                    },
                    MaterialPropertyKind::Scalar => match value {
                        VoxValuePoolValueRef::Float(number) => {
                            check_material_range(name, number, range)?
                        }
                        VoxValuePoolValueRef::Int(number) => {
                            check_material_range(name, number as f64, range)?
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}

/// Errors unless every component of a vocabulary color lies in `range`.
fn check_components(name: &str, components: &[f64], range: MaterialRange) -> Result<()> {
    for &component in components {
        check_material_range(name, component, range)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::utilities::check_material_property_ranges;
    use branded_id::U32Id;
    use voxcore::{
        VoxMain, VoxPalette, VoxValuePool,
        material::{BASE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC},
    };

    /// A main whose one palette binds `name` to a one-material value pool.
    fn main_with(name: &str, value_pool: VoxValuePool) -> VoxMain {
        let mut main = VoxMain::default();
        let value_pool_id = main.retain_value_pool(value_pool);

        let mut palette = VoxPalette::default();
        palette
            .retain_property(name.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        main.retain_palette(palette).unwrap();

        main
    }

    #[test]
    fn passes_in_range_values_and_unranged_names() {
        let main = main_with(METALLIC, VoxValuePool::float(vec![1.0]).unwrap());
        assert!(check_material_property_ranges(&main).is_ok());

        // `emissiveStrength` is unbounded above.
        let main = main_with(EMISSIVE_STRENGTH, VoxValuePool::float(vec![7.0]).unwrap());
        assert!(check_material_property_ranges(&main).is_ok());

        // A custom name has no checkable range.
        let main = main_with("subsurface", VoxValuePool::float(vec![7.0]).unwrap());
        assert!(check_material_property_ranges(&main).is_ok());
    }

    #[test]
    fn errors_on_a_scalar_outside_its_range() {
        let main = main_with(METALLIC, VoxValuePool::float(vec![1.5]).unwrap());
        let message = check_material_property_ranges(&main)
            .unwrap_err()
            .to_string();
        assert!(message.contains(METALLIC), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn holds_the_ior_union_exactly() {
        // Zero means "does not refract" and passes. A value between the
        // union's parts rejects.
        let main = main_with(IOR, VoxValuePool::float(vec![0.0, 1.5]).unwrap());
        assert!(check_material_property_ranges(&main).is_ok());

        let main = main_with(IOR, VoxValuePool::float(vec![0.5]).unwrap());
        let message = check_material_property_ranges(&main)
            .unwrap_err()
            .to_string();
        assert!(message.contains(IOR), "{message}");
    }

    #[test]
    fn errors_on_a_color_component_outside_zero_to_one() {
        let main = main_with(
            BASE_COLOR,
            VoxValuePool::vec_4_float(vec![[1.5, 0.0, 0.0, 1.0]]).unwrap(),
        );
        let message = check_material_property_ranges(&main)
            .unwrap_err()
            .to_string();
        assert!(message.contains(BASE_COLOR), "{message}");
    }

    #[test]
    fn skips_a_value_no_material_draws() {
        // No material draws the out-of-range second value, so it passes.
        let main = main_with(METALLIC, VoxValuePool::float(vec![1.0, 7.0]).unwrap());
        assert!(check_material_property_ranges(&main).is_ok());
    }
}
