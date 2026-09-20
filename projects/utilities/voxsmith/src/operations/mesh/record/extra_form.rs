/// An extras entry's form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtraForm {
    /// The entry references a texture.
    Image,

    /// The entry holds JSON.
    Json,
}
