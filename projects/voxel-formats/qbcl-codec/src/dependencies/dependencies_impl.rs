use crate::{CompressZlib, DecompressZlib};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

/// The dependencies over `flate2`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl CompressZlib for DependenciesImpl {
    fn compress_zlib(&self, bytes: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(bytes)
            .expect("write to Vec is infallible");
        encoder.finish().expect("flush to Vec is infallible")
    }
}

impl DecompressZlib for DependenciesImpl {
    fn decompress_zlib(&self, stream: &[u8]) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        ZlibDecoder::new(stream)
            .read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{CompressZlib, DecompressZlib, DependenciesImpl};

    #[test]
    fn zlib_round_trips() {
        let bytes: Vec<u8> = (0..4096u32).map(|i| (i % 7) as u8).collect();
        let stream = DependenciesImpl.compress_zlib(&bytes);
        assert!(stream.len() < bytes.len());
        assert_eq!(DependenciesImpl.decompress_zlib(&stream).unwrap(), bytes);
    }

    #[test]
    fn decompress_rejects_a_malformed_stream() {
        assert!(
            DependenciesImpl
                .decompress_zlib(&[0xFF, 0xFF, 0xFF])
                .is_err()
        );
    }
}
