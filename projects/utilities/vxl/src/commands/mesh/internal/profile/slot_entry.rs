use serde::Deserialize;

/// A profile's material slot source, its kind the tail of the
/// `--write-material-slot-*` flag it mirrors.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum SlotEntry {
    File { file: String },
    Value { value: String },
}
