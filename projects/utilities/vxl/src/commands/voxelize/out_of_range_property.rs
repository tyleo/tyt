use crate::CliValue;
use voxsmith::OutOfRangeProperty;

impl CliValue for OutOfRangeProperty {
    const VARIANTS: &'static [Self] = &[OutOfRangeProperty::Error, OutOfRangeProperty::Clamp];

    fn name(self) -> &'static str {
        match self {
            OutOfRangeProperty::Error => "error",
            OutOfRangeProperty::Clamp => "clamp",
        }
    }

    fn help(self) -> &'static str {
        match self {
            OutOfRangeProperty::Error => "Reject the mesh, naming the property and its value",
            OutOfRangeProperty::Clamp => "Clamp the value onto the range and voxelize on",
        }
    }
}
