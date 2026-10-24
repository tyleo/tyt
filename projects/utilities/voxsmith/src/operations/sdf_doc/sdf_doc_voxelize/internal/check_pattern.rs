use crate::operations::sdf_doc::ArgumentCheck;
use sdfcore::SdfPattern;
use std::result::Result as StdResult;

/// Checks the arguments of the pattern `pattern`.
pub fn check_pattern(pattern: &SdfPattern) -> StdResult<(), String> {
    match pattern {
        SdfPattern::Bands {
            material_ids,
            period,
            warp,
            seed,
            ..
        } => {
            let check = ArgumentCheck::new("bands");
            check.count("materials", material_ids.len(), 1)?;
            check.above_zero("period", *period)?;
            optional_at_least_zero(check, "warp", *warp)?;
            optional_seed(check, *seed)
        }

        SdfPattern::Bricks {
            material_ids,
            length,
            height,
            seed,
            border,
            ..
        } => {
            let check = ArgumentCheck::new("bricks");
            check.count("materials", material_ids.len(), 1)?;
            check.above_zero("length", *length)?;
            check.above_zero("height", *height)?;

            if let Some(border) = border {
                // A border as wide as a brick fills the whole pattern.
                check.above_zero("border width", border.width)?;
                check.expect(
                    border.width < *height,
                    "border width",
                    &format!("below height {height}"),
                    border.width,
                )?;
                check.expect(
                    border.width < *length,
                    "border width",
                    &format!("below length {length}"),
                    border.width,
                )?;
            }

            check.seed(*seed)
        }

        SdfPattern::Cells {
            material_ids,
            size,
            seed,
            border,
        } => {
            let check = ArgumentCheck::new("cells");
            check.count("materials", material_ids.len(), 1)?;
            check.above_zero("size", *size)?;

            if let Some(border) = border {
                check.above_zero("border width", border.width)?;
            }

            check.seed(*seed)
        }

        SdfPattern::Checker { material_ids, size } => {
            let check = ArgumentCheck::new("checker");
            check.count("materials", material_ids.len(), 1)?;
            check.above_zero("size", *size)
        }

        SdfPattern::Gradient {
            material_ids,
            from,
            to,
            warp,
            seed,
            ..
        } => {
            let check = ArgumentCheck::new("gradient");
            check.count("materials", material_ids.len(), 1)?;
            check.range(*from, *to)?;
            optional_at_least_zero(check, "warp", *warp)?;
            optional_seed(check, *seed)
        }

        SdfPattern::Grain {
            material_ids,
            period,
            warp,
            seed,
            ..
        } => {
            let check = ArgumentCheck::new("grain");
            check.count("materials", material_ids.len(), 1)?;
            check.above_zero("period", *period)?;
            optional_at_least_zero(check, "warp", *warp)?;
            check.seed(*seed)
        }

        SdfPattern::Noise {
            material_ids,
            scale,
            octaves,
            seed,
        } => {
            let check = ArgumentCheck::new("noise");
            check.count("materials", material_ids.len(), 1)?;
            check.above_zero("scale", *scale)?;

            if let Some(octaves) = octaves {
                check.whole_above_zero("octaves", *octaves)?;
            }

            check.seed(*seed)
        }

        SdfPattern::Speckle {
            accent_ids,
            size,
            density,
            seed,
            ..
        } => {
            let check = ArgumentCheck::new("speckle");
            check.count("accents", accent_ids.len(), 1)?;
            check.above_zero("size", *size)?;
            check.expect(
                (0.0..=1.0).contains(density),
                "density",
                "from 0 to 1",
                density,
            )?;
            check.seed(*seed)
        }
    }
}

/// Errors unless `value`, when given, reads zero or more.
fn optional_at_least_zero(
    check: ArgumentCheck,
    argument: &str,
    value: Option<f64>,
) -> StdResult<(), String> {
    value.map_or(Ok(()), |value| check.at_least_zero(argument, value))
}

/// Errors unless `seed`, when given, fits the hash.
fn optional_seed(check: ArgumentCheck, seed: Option<f64>) -> StdResult<(), String> {
    seed.map_or(Ok(()), |seed| check.seed(seed))
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::check_pattern;
    use branded_id::U32Id;
    use sdfcore::{SdfBorder, SdfPattern};

    #[test]
    fn a_speckle_takes_a_size_above_zero_a_density_from_0_to_1_and_a_whole_seed() {
        let speckle = |size, density, seed| SdfPattern::Speckle {
            base_id: U32Id::from_u32(0),
            accent_ids: vec![U32Id::from_u32(1)],
            size,
            density,
            seed,
        };

        assert!(check_pattern(&speckle(0.1, 1.0, -7.0)).is_ok());
        assert_eq!(
            check_pattern(&speckle(0.0, 0.5, 7.0)),
            Err("speckle size must be above zero, not 0".to_string())
        );
        assert_eq!(
            check_pattern(&speckle(0.1, 1.5, 7.0)),
            Err("speckle density must be from 0 to 1, not 1.5".to_string())
        );
        assert_eq!(
            check_pattern(&speckle(0.1, 0.5, 0.5)),
            Err("speckle seed must be a whole number from -2^31 to 2^31 - 1, not 0.5".to_string())
        );
    }

    #[test]
    fn a_bricks_border_takes_a_width_above_zero_and_below_the_brick() {
        let bricks = |width| SdfPattern::Bricks {
            material_ids: vec![U32Id::from_u32(0)],
            length: 0.2,
            height: 0.06,
            axis: None,
            seed: 1.0,
            border: Some(SdfBorder {
                material_id: U32Id::from_u32(1),
                width,
            }),
        };

        assert!(check_pattern(&bricks(0.01)).is_ok());
        assert_eq!(
            check_pattern(&bricks(0.0)),
            Err("bricks border width must be above zero, not 0".to_string())
        );
        assert_eq!(
            check_pattern(&bricks(0.06)),
            Err("bricks border width must be below height 0.06, not 0.06".to_string())
        );
    }

    #[test]
    fn a_cells_border_takes_a_width_above_zero() {
        let cells = |width| SdfPattern::Cells {
            material_ids: vec![U32Id::from_u32(0)],
            size: 0.2,
            seed: 1.0,
            border: Some(SdfBorder {
                material_id: U32Id::from_u32(1),
                width,
            }),
        };

        assert!(check_pattern(&cells(0.02)).is_ok());
        assert_eq!(
            check_pattern(&cells(0.0)),
            Err("cells border width must be above zero, not 0".to_string())
        );
    }
}
