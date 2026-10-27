/// One file of the builder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SdfjBuilderFile {
    /// The file's path in the builder's directory.
    pub path: &'static str,

    /// The file's contents.
    pub text: &'static str,
}
