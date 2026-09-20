use crate::CliValue;
use serde::{Deserialize, Deserializer, de::Error as DeError};

/// A [`CliValue`] a profile writes by the name the command line takes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NamedCliValue<T>(pub(crate) T);

impl<'de, T: CliValue> Deserialize<'de> for NamedCliValue<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;

        T::parse(&name).map(NamedCliValue).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::NamedCliValue;
    use voxsmith::operations::mesh::Transfer;

    #[test]
    fn a_value_reads_by_its_command_line_name() {
        assert_eq!(
            serde_json::from_str::<NamedCliValue<Transfer>>("\"srgb\"").unwrap(),
            NamedCliValue(Transfer::Srgb)
        );
        assert!(serde_json::from_str::<NamedCliValue<Transfer>>("\"gamma\"").is_err());
    }
}
