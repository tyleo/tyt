use crate::NamedCliValue;
use serde::Deserialize;
use voxsmith::operations::object::Transfer;

/// A profile's written value, an expression with its transfer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ValueEntry {
    pub(crate) transfer: NamedCliValue<Transfer>,

    pub(crate) value: String,
}
