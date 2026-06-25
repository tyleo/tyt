use crate::{Error, Result};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

/// Decompresses a zlib stream, mapping a malformed one to [`Error::Invalid`].
pub fn zlib_decompress(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|error| Error::Invalid(format!("zlib decompression failed: {error}")))?;
    Ok(out)
}

/// Compresses bytes into a zlib stream.
pub fn zlib_compress(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .expect("writing to an in-memory buffer is infallible");
    encoder
        .finish()
        .expect("finishing an in-memory buffer is infallible")
}
