use crate::to_voxj_bytes;
use flate2::{Compression, Crc, write::DeflateEncoder};
use std::io::Write;
use voxj::VoxjFile;

/// Conventional name of the single `.voxj` member inside a `.voxjz` archive.
const MEMBER_NAME: &[u8] = b"main.voxj";

/// Serializes a document to `.voxjz` bytes: a zip archive holding one
/// deflate-compressed `main.voxj` member whose content is byte-identical to a
/// standalone `.voxj`.
pub fn to_voxjz_bytes(file: &VoxjFile) -> serde_json::Result<Vec<u8>> {
    let member = to_voxj_bytes(file, false)?;

    let mut crc = Crc::new();
    crc.update(&member);
    let crc = crc.sum();

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(&member)
        .expect("write to Vec is infallible");
    let compressed = encoder.finish().expect("flush to Vec is infallible");

    let name_len = MEMBER_NAME.len() as u16;
    let mut out = Vec::new();

    // Local file header.
    out.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // signature
    out.extend_from_slice(&20u16.to_le_bytes()); // version needed
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&8u16.to_le_bytes()); // method: deflate
    out.extend_from_slice(&0u16.to_le_bytes()); // mod time
    out.extend_from_slice(&0u16.to_le_bytes()); // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(member.len() as u32).to_le_bytes());
    out.extend_from_slice(&name_len.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // extra length
    out.extend_from_slice(MEMBER_NAME);
    out.extend_from_slice(&compressed);

    // Central directory header.
    let cd_offset = out.len() as u32;
    out.extend_from_slice(&0x0201_4b50u32.to_le_bytes()); // signature
    out.extend_from_slice(&20u16.to_le_bytes()); // version made by
    out.extend_from_slice(&20u16.to_le_bytes()); // version needed
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&8u16.to_le_bytes()); // method
    out.extend_from_slice(&0u16.to_le_bytes()); // mod time
    out.extend_from_slice(&0u16.to_le_bytes()); // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(member.len() as u32).to_le_bytes());
    out.extend_from_slice(&name_len.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // extra length
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length
    out.extend_from_slice(&0u16.to_le_bytes()); // disk number start
    out.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
    out.extend_from_slice(&0u32.to_le_bytes()); // external attributes
    out.extend_from_slice(&0u32.to_le_bytes()); // local header offset
    out.extend_from_slice(MEMBER_NAME);
    let cd_size = out.len() as u32 - cd_offset;

    // End of central directory.
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // signature
    out.extend_from_slice(&0u16.to_le_bytes()); // this disk
    out.extend_from_slice(&0u16.to_le_bytes()); // cd start disk
    out.extend_from_slice(&1u16.to_le_bytes()); // entries this disk
    out.extend_from_slice(&1u16.to_le_bytes()); // total entries
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length

    Ok(out)
}
