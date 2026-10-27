/// A font file a document's font points to: its bytes and the face to read.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfjFontFile {
    /// The font file's bytes.
    pub data: Vec<u8>,

    /// The face's index in the file, 0 for a file holding one face.
    pub index: u32,
}
