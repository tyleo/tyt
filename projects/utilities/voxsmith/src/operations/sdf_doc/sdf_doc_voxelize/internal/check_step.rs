use crate::operations::sdf_doc::ArgumentCheck;
use branded_id::U32Id;
use sdfcore::{BSdfStep, SdfStep};
use std::{collections::HashSet, result::Result as StdResult};

/// Checks the arguments of the step `step` other than its shapes, material,
/// and pattern.
pub fn check_step(step: &SdfStep) -> StdResult<(), String> {
    match step {
        SdfStep::Add { .. } | SdfStep::Carve { .. } => Ok(()),

        SdfStep::Coat {
            depth, onto_ids, ..
        } => {
            let check = ArgumentCheck::new("coat");
            check.above_zero("depth", *depth)?;
            check_onto(&check, onto_ids.as_deref())
        }

        SdfStep::Paint { onto_ids, .. } => {
            check_onto(&ArgumentCheck::new("paint"), onto_ids.as_deref())
        }

        SdfStep::Set { points, size, .. } => {
            let check = ArgumentCheck::new("set");
            check.above_zero("size", *size)?;

            let mut seen = HashSet::with_capacity(points.len());

            match points
                .iter()
                .find(|point| !seen.insert(point.to_array().map(f64::to_bits)))
            {
                Some(point) => Err(check.fail(
                    "points",
                    "distinct",
                    format!("[{}, {}, {}] twice", point.x, point.y, point.z),
                )),

                None => Ok(()),
            }
        }
    }
}

/// Errors on an `onto` that lists no step.
fn check_onto(
    check: &ArgumentCheck,
    onto_ids: Option<&[U32Id<BSdfStep>]>,
) -> StdResult<(), String> {
    match onto_ids {
        Some(onto_ids) => check.count("onto", onto_ids.len(), 1),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::check_step;
    use branded_id::U32Id;
    use sdfcore::{SdfStep, SdfStepMaterial};
    use ty_math::TyVector3F64;

    #[test]
    fn a_set_lists_each_point_once_at_a_size_a_coat_reaches_past_zero_and_onto_lists_a_step() {
        let set = |points, size| SdfStep::Set {
            name: "studs".to_string(),
            points,
            size,
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
        };

        assert!(check_step(&set(vec![TyVector3F64::ZERO, TyVector3F64::X], 0.1)).is_ok());
        assert_eq!(
            check_step(&set(
                vec![TyVector3F64::X, TyVector3F64::ZERO, TyVector3F64::X],
                0.1
            )),
            Err("set points must be distinct, not [1, 0, 0] twice".to_string())
        );
        assert_eq!(
            check_step(&set(vec![TyVector3F64::ZERO], 0.0)),
            Err("set size must be above zero, not 0".to_string())
        );

        let coat = SdfStep::Coat {
            name: "skin".to_string(),
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            sides: None,
            depth: 0.0,
            within_id: None,
            onto_ids: None,
        };
        assert_eq!(
            check_step(&coat),
            Err("coat depth must be above zero, not 0".to_string())
        );

        let paint = SdfStep::Paint {
            name: "gilt".to_string(),
            shape_id: U32Id::from_u32(0),
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
            onto_ids: Some(Vec::new()),
        };
        assert_eq!(
            check_step(&paint),
            Err("paint onto must be a list of at least 1, not a list of 0".to_string())
        );
    }
}
