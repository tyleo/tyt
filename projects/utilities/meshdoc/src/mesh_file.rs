/// A loose file that lands beside the document's primary file. An image can
/// read its bytes from a file, and a property can point at one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshFile {
    /// The relative name a writer lands the file under, unique within the
    /// document, such as `atlas.png` or `values/albedo.json`.
    pub name: String,

    /// The file's bytes.
    pub bytes: Vec<u8>,
}
