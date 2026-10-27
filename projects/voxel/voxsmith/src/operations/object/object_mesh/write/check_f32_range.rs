use crate::{Error, Result, operations::object::MeshElement};

/// Passes the finite `component` through, erroring where it rounds past the
/// `f32` range. A glTF float holds `f32` alone, and the glTF writer rounds
/// each one to the nearest.
pub fn check_f32_range(element: &MeshElement, component: f64) -> Result<f64> {
    if (component as f32).is_finite() {
        Ok(component)
    } else {
        Err(Error::mesh_record(
            element.clone(),
            format!("holds {component}, past the f32 range of a glTF float"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{MeshElement, check_f32_range};

    fn element() -> MeshElement {
        MeshElement::File {
            file: "bar.json".to_owned(),
        }
    }

    #[test]
    fn a_component_past_the_f32_range_errors() {
        let largest = f64::from(f32::MAX);

        assert_eq!(check_f32_range(&element(), 0.1).unwrap(), 0.1);
        assert_eq!(check_f32_range(&element(), -largest).unwrap(), -largest);
        assert!(check_f32_range(&element(), 1e39).is_err());
        assert!(check_f32_range(&element(), -1e39).is_err());
    }
}
