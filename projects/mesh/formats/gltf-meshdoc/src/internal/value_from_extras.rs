use crate::{Error, Result};
use gltf::json::Extras;
use serde_json::Value;

/// An object's `extras` as a JSON value, or `None` when it has none.
pub fn value_from_extras(extras: &Extras) -> Result<Option<Value>> {
    extras
        .as_ref()
        .map(|raw| {
            serde_json::from_str(raw.get())
                .map_err(|error| Error::invalid(format!("extras are not JSON: {error}")))
        })
        .transpose()
}
