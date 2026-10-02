use crate::{Result, utilities::check_material_range};
use branded_id::U32Id;
use meshdoc::material::{COLOR_RANGE, MaterialPropertyKind, scalar_range};
use voxcore::{BVoxValuePoolValue, VoxExt, VoxMain, VoxValueColumn, VoxValuePoolValues};

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
    for (_, palette) in main.iter_palettes() {
        for (property_id, property) in palette.iter_properties() {
            let name = &property.name;
            let Some(kind) = MaterialPropertyKind::of(name) else {
                continue;
            };

            let value_pool = main
                .value_pool(property.value_pool_id)
                .expect("a property names a live value pool");

            let value_ids = palette.iter_materials().map(|material_id| {
                palette
                    .value_id(material_id, property_id)
                    .expect("a material has a value id for every property")
            });

            match (kind, value_pool.values()) {
                (
                    MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba,
                    VoxValuePoolValues::Vec3Float(colors),
                ) => check_colors(name, colors, value_ids)?,

                (
                    MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba,
                    VoxValuePoolValues::Vec4Float(colors),
                ) => check_colors(name, colors, value_ids)?,

                (MaterialPropertyKind::Scalar, VoxValuePoolValues::Float(numbers)) => {
                    let range =
                        scalar_range(name).expect("every scalar vocabulary property has a range");

                    for value_id in value_ids {
                        check_material_range(name, *drawn(numbers, value_id), range)?;
                    }
                }

                (MaterialPropertyKind::Scalar, VoxValuePoolValues::Int(numbers)) => {
                    let range =
                        scalar_range(name).expect("every scalar vocabulary property has a range");

                    for value_id in value_ids {
                        check_material_range(name, *drawn(numbers, value_id) as f64, range)?;
                    }
                }

                _ => {}
            }
        }
    }

    Ok(())
}

/// Errors unless every component of each color `value_ids` draws lies in the
/// color range.
fn check_colors<const N: usize>(
    name: &str,
    colors: VoxValueColumn<'_, [f64; N]>,
    value_ids: impl Iterator<Item = U32Id<BVoxValuePoolValue>>,
) -> Result<()> {
    for value_id in value_ids {
        for &component in drawn(colors, value_id) {
            check_material_range(name, component, COLOR_RANGE)?;
        }
    }

    Ok(())
}

fn drawn<'a, T>(values: VoxValueColumn<'a, T>, value_id: U32Id<BVoxValuePoolValue>) -> &'a T {
    values
        .get(value_id)
        .expect("a material draws one of its property's values")
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
