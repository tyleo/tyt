use crate::{Error, Result};

/// The whole-number factor a resample scales an object's grid by on every
/// axis, at least 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResampleFactor(u32);

impl ResampleFactor {
    /// Errors below 2.
    pub fn new(factor: u32) -> Result<Self> {
        if factor < 2 {
            return Err(Error::invalid(format!(
                "a resample factor is at least 2, not {factor}"
            )));
        }

        Ok(Self(factor))
    }

    /// The factor.
    pub fn get(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::object::ResampleFactor;

    #[test]
    fn a_factor_is_at_least_two() {
        assert_eq!(ResampleFactor::new(2).unwrap().get(), 2);
        assert!(ResampleFactor::new(1).is_err());
        assert!(ResampleFactor::new(0).is_err());
    }
}
