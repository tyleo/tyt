use crate::{Error, Result};

/// The entry for an entity, or the error for an ext out of step with the
/// scene.
pub(crate) fn ext_entry<'a, T>(entry: Option<&'a T>, what: &str, id: u32) -> Result<&'a T> {
    entry.ok_or_else(|| Error::invalid(format!("vmax ext holds no entry for {what} {id}")))
}
