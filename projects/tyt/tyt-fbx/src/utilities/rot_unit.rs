use clap::ValueEnum;

/// Unit used for rotation-valued CLI inputs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum RotUnit {
    #[default]
    Rad,
    Deg,
}

impl RotUnit {
    /// Converts `value` from this unit to radians.
    pub fn to_radians(self, value: f64) -> f64 {
        match self {
            RotUnit::Rad => value,
            RotUnit::Deg => value.to_radians(),
        }
    }
}
