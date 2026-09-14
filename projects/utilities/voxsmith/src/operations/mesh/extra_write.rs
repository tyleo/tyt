use crate::operations::mesh::{ExtraForm, ExtraSource};

/// A named property of a material or the object, which the glTF bridge
/// writes under `extras.vxl.values`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtraWrite {
    /// The entry's name.
    pub name: String,

    /// The entry's form.
    pub form: ExtraForm,

    /// The entry's source.
    pub source: ExtraSource,
}
