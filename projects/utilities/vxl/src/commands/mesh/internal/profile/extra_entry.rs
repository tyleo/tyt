use crate::commands::NamedCliValue;
use serde::Deserialize;
use voxsmith::operations::mesh::Transfer;

/// A profile's extras entry, its kind the tail of the extras flag it mirrors.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ExtraEntry {
    ImageFile {
        file: String,
    },

    ImageValue {
        transfer: NamedCliValue<Transfer>,
        value: String,
    },

    JsonFile {
        file: String,
    },

    JsonValue {
        transfer: NamedCliValue<Transfer>,
        value: String,
    },
}
