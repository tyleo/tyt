use crate::commands::ValueEntry;
use serde::Deserialize;
use std::collections::BTreeMap;

/// A profile's primitive, its list position the flags' primitive index.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PrimitiveEntry {
    /// Mirrors `--primitive-name`.
    pub(crate) name: Option<String>,

    /// Mirrors the `--primitive` select argument. Absent, `true`.
    pub(crate) select: Option<String>,

    /// Mirrors the `--primitive` material index. Absent, `none`.
    pub(crate) material: Option<u32>,

    /// Mirrors `--write-primitive-normal`. Absent, `true`.
    pub(crate) normal: Option<bool>,

    /// Mirrors `--write-primitive-uv` per entry. Absent, the material's list.
    pub(crate) uvs: Option<Vec<String>>,

    /// Mirrors `--write-primitive-builtin-value` per attribute.
    pub(crate) builtins: BTreeMap<String, String>,

    /// Mirrors `--write-primitive-custom-value` per underscore name.
    pub(crate) customs: BTreeMap<String, ValueEntry>,
}
