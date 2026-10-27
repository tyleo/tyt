use crate::operations::sdf_doc::ArgumentCheck;
use sdfcore::SdfShades;
use std::result::Result as StdResult;

/// Checks the arguments of the `shades` call `shades`.
pub fn check_shades(shades: &SdfShades) -> StdResult<(), String> {
    let check = ArgumentCheck::new("shades");
    check.whole_above_zero("count", shades.count)?;

    match shades.spread {
        Some(spread) => check.above_zero("spread", spread),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::check_shades;
    use branded_id::U32Id;
    use sdfcore::SdfShades;

    /// The error a call of `count` shades `spread` apart reads.
    fn error(count: f64, spread: Option<f64>) -> Option<String> {
        check_shades(&SdfShades {
            base_id: U32Id::from_u32(0),
            count,
            spread,
        })
        .err()
    }

    #[test]
    fn the_count_is_whole_and_the_spread_above_zero() {
        assert_eq!(error(3.0, None), None);
        assert_eq!(
            error(2.5, None).as_deref(),
            Some("shades count must be a whole number above zero, not 2.5")
        );
        assert_eq!(
            error(3.0, Some(0.0)).as_deref(),
            Some("shades spread must be above zero, not 0")
        );
    }
}
