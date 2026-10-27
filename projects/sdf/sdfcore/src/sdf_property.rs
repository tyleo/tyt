use crate::SdfPropertyValue;

/// One property of a material.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfProperty {
    /// The property's name: a glTF vocabulary name or a custom one.
    pub name: String,

    /// The property's value.
    pub value: SdfPropertyValue,
}
