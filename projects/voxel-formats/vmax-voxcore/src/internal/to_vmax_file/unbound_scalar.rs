use voxcore::material::default_scalar;

/// `value` when the property is bound, else the glTF vocabulary default the
/// format gives `key`, so an unbound property writes what it renders as.
pub(crate) fn unbound_scalar(value: Option<f64>, key: &str) -> f64 {
    value.unwrap_or_else(|| {
        default_scalar(key).expect("a vocabulary scalar the vmax writer emits has a spec default")
    })
}
