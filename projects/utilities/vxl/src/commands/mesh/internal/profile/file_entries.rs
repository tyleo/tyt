use crate::commands::ValueEntry;
use serde::Deserialize;
use std::collections::BTreeMap;

/// A profile's `files`, the writes beside the mesh keyed by file template.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct FileEntries {
    /// The JSON files, each holding its values by key.
    pub(crate) json: BTreeMap<String, BTreeMap<String, ValueEntry>>,

    /// The PNG files, each holding one array value.
    pub(crate) png: BTreeMap<String, ValueEntry>,
}
