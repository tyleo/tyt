use crate::commands::{ExtraEntry, SlotEntry};
use serde::Deserialize;
use std::collections::BTreeMap;

/// A profile's material, its list position the flags' material index.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct MaterialEntry {
    /// Mirrors `--material-name`.
    pub(crate) name: Option<String>,

    /// Mirrors `--material-uv` per entry. Absent, the list derives from use.
    pub(crate) uvs: Option<Vec<String>>,

    /// Mirrors the `--write-material-slot-*` flags, a property per key.
    pub(crate) slots: BTreeMap<String, SlotEntry>,

    /// Mirrors the `--write-material-extra-*` flags, an entry per name.
    pub(crate) extras: BTreeMap<String, ExtraEntry>,
}
