use crate::{Components, Value};

/// Asserts two `f64` lists agree within a small tolerance.
pub fn assert_close(found: &Value, expected: &[f64]) {
    let Components::F64(found) = found.components() else {
        panic!("{found:?} is not f64");
    };

    assert_eq!(
        found.len(),
        expected.len(),
        "{found:?} against {expected:?}"
    );

    for (found, expected) in found.iter().zip(expected) {
        assert!(
            (found - expected).abs() <= 1e-5 * expected.abs().max(1.0),
            "{found} is not {expected}"
        );
    }
}
