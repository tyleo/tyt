use crate::{Error, Result};
use meshdoc::material::MaterialRange;

/// Errors unless `value` lies in `range`. The error reports the vocabulary
/// property `name`.
pub fn check_material_range(name: &str, value: f64, range: MaterialRange) -> Result<()> {
    if range.contains(value) {
        Ok(())
    } else {
        Err(Error::invalid(format!(
            "`{name}` is {value}, outside the range {range}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::check_material_range;
    use meshdoc::material::{IOR, scalar_range};

    #[test]
    fn names_the_property_and_the_range() {
        let range = scalar_range(IOR).unwrap();

        assert!(check_material_range(IOR, 0.0, range).is_ok());

        let message = check_material_range(IOR, 0.5, range)
            .unwrap_err()
            .to_string();
        assert!(message.contains(IOR), "{message}");
        assert!(message.contains("0, or 1 or greater"), "{message}");
    }
}
