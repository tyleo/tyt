use crate::operations::mesh::{FileForm, WrittenValue};

/// A file the run lands in the document, written beside the mesh.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileWrite {
    /// The file's relative name.
    pub file: String,

    /// The written value.
    pub value: WrittenValue,

    /// The file's form.
    pub form: FileForm,
}
