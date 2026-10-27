use sdfcore::SdfValue;

/// A custom property's value in one material, in the voxj kind the value takes.
#[derive(Clone, Debug, PartialEq)]
pub enum SdfCustomValue {
    /// A `bool`.
    Bool(bool),

    /// A `float`.
    Float(f64),

    /// A `vec-N-float` of 2 to 4 numbers.
    FloatVector(Vec<f64>),

    /// An `int` within `2^53 - 1`.
    Int(i64),

    /// A `vec-N-int` of 2 to 4 whole numbers within `2^53 - 1`.
    IntVector(Vec<i64>),

    /// A `json` value.
    Json(SdfValue),

    /// A `string`.
    Text(String),
}

impl SdfCustomValue {
    /// The voxj kind of the value, such as `vec-3-float`.
    pub fn kind(&self) -> String {
        match self {
            Self::Bool(_) => "bool".to_owned(),
            Self::Float(_) => "float".to_owned(),
            Self::FloatVector(values) => format!("vec-{}-float", values.len()),
            Self::Int(_) => "int".to_owned(),
            Self::IntVector(values) => format!("vec-{}-int", values.len()),
            Self::Json(_) => "json".to_owned(),
            Self::Text(_) => "string".to_owned(),
        }
    }

    /// The empty value of the value's kind: 0, a zero vector, `false`, `""`,
    /// or `null`.
    pub fn empty(&self) -> Self {
        match self {
            Self::Bool(_) => Self::Bool(false),
            Self::Float(_) => Self::Float(0.0),
            Self::FloatVector(values) => Self::FloatVector(vec![0.0; values.len()]),
            Self::Int(_) => Self::Int(0),
            Self::IntVector(values) => Self::IntVector(vec![0; values.len()]),
            Self::Json(_) => Self::Json(SdfValue::Null),
            Self::Text(_) => Self::Text(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::SdfCustomValue;
    use sdfcore::SdfValue;

    #[test]
    fn a_vector_kind_counts_its_components_and_empties_to_zeros() {
        let value = SdfCustomValue::IntVector(vec![3, -1, 7]);

        assert_eq!(value.kind(), "vec-3-int");
        assert_eq!(value.empty(), SdfCustomValue::IntVector(vec![0, 0, 0]));
    }

    #[test]
    fn each_kind_empties_to_its_empty_value() {
        assert_eq!(
            SdfCustomValue::Bool(true).empty(),
            SdfCustomValue::Bool(false)
        );
        assert_eq!(
            SdfCustomValue::Json(SdfValue::Number(2.0)).empty(),
            SdfCustomValue::Json(SdfValue::Null)
        );
        assert_eq!(
            SdfCustomValue::Text("oak".to_owned()).empty(),
            SdfCustomValue::Text(String::new())
        );
    }
}
