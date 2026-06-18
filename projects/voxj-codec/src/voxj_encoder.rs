use crate::wrap_voxjz;
use serde_json::Value;
use voxj::VoxjFile;

/// Builds `.voxj` / `.voxjz` bytes for a document, optionally embedding an opaque
/// extension blob at `main.ext`.
///
/// ```ignore
/// let json = VoxjEncoder::new(&file).json()?;            // no extension data
/// let zip = VoxjEncoder::new(&file).ext(&ext).zip()?;   // with extension data
/// ```
pub struct VoxjEncoder<'a> {
    file: &'a VoxjFile,
    ext: Option<&'a [u8]>,
}

impl<'a> VoxjEncoder<'a> {
    /// Starts an encoder for `file` with no extension data.
    pub fn new(file: &'a VoxjFile) -> Self {
        Self { file, ext: None }
    }

    /// Embeds `ext` (raw UTF-8 JSON) verbatim at `main.ext`. The codec ascribes
    /// no meaning to it; tools layer their own format-specific data under named
    /// keys.
    pub fn ext(mut self, ext: &'a [u8]) -> Self {
        self.ext = Some(ext);
        self
    }

    /// Serializes to compact `.voxj` JSON.
    pub fn json(&self) -> serde_json::Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec(&self.document()?)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Serializes to pretty-printed `.voxj` JSON.
    pub fn pretty(&self) -> serde_json::Result<Vec<u8>> {
        let mut bytes = serde_json::to_vec_pretty(&self.document()?)?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Serializes to a `.voxjz` zip archive holding one compact `.voxj` member.
    pub fn zip(&self) -> serde_json::Result<Vec<u8>> {
        Ok(wrap_voxjz(&self.json()?))
    }

    /// The document as a JSON value, with `ext` inserted at `main.ext` when
    /// present and non-empty.
    fn document(&self) -> serde_json::Result<Value> {
        let mut value = serde_json::to_value(self.file)?;
        if let Some(ext) = self.ext.filter(|ext| !ext.is_empty())
            && let Some(main) = value.get_mut("main").and_then(Value::as_object_mut)
        {
            main.insert("ext".to_owned(), serde_json::from_slice(ext)?);
        }
        Ok(value)
    }
}
