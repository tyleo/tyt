use crate::{PositionEncoding, SampleEncoding};

/// How [`encode_object`](crate::encode_object) chooses block encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodingChoice {
    /// Use exactly these position and sample encodings.
    Fixed {
        position: PositionEncoding,
        sample: SampleEncoding,
    },
    /// Build the matrix of non-raw encodings, deflate each pairing, and keep the
    /// smallest. The canonical shipping form.
    Smallest,
}
