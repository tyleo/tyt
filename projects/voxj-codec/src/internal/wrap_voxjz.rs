use flate2::{Compression, Crc, write::DeflateEncoder};
use std::io::Write;

/// Conventional name of the single `.voxj` member inside a `.voxjz` archive.
const MEMBER_NAME: &[u8] = b"main.voxj";

/// Value stored in a 32-bit size or offset field to mean "the real 64-bit value
/// lives in the zip64 record". A true value at or above this cannot be stored
/// inline, so the archive must switch to zip64 framing.
const ZIP64_SENTINEL: u32 = 0xFFFF_FFFF;

/// "Version needed to extract" advertised by records that use zip64 (4.5).
const ZIP64_VERSION: u16 = 45;

/// "Version needed to extract" advertised by classic, non-zip64 records (2.0).
const BASE_VERSION: u16 = 20;

/// Fixed DOS modification time and date stamped on the member so archives stay
/// reproducible (byte-identical for identical input) rather than varying with
/// wall-clock time. `1980-01-01 00:00:00` is the earliest the DOS format can
/// represent: date = (year - 1980) << 9 | month << 5 | day = (1 << 5) | 1 =
/// 0x0021, time = 0. An all-zero date is the invalid `day 0, month 0` that some
/// tools flag.
const DOS_TIME: u16 = 0x0000;
const DOS_DATE: u16 = 0x0021;

/// Wraps a `.voxj` byte payload in a single-member, deflate-compressed `.voxjz`
/// zip archive. The common case uses classic 32-bit framing; the writer
/// transparently switches to zip64 when the member's compressed or uncompressed
/// size, or the central-directory offset, would overflow a 32-bit field
/// (>= 4 GiB), so a large payload still yields a valid archive instead of one
/// with silently truncated sizes.
pub(crate) fn wrap_voxjz(member: &[u8]) -> Vec<u8> {
    wrap_voxjz_with(member, u64::from(ZIP64_SENTINEL))
}

/// Builds the archive, routing any size or offset that reaches `zip64_at`
/// through zip64 framing. Production passes `0xFFFFFFFF` (the one value a 32-bit
/// field can never store inline); tests pass a small value to drive the zip64
/// path with a tiny payload.
fn wrap_voxjz_with(member: &[u8], zip64_at: u64) -> Vec<u8> {
    let mut crc = Crc::new();
    crc.update(member);
    let crc = crc.sum();

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(member)
        .expect("write to Vec is infallible");
    let compressed = encoder.finish().expect("flush to Vec is infallible");

    let uncompressed_size = member.len() as u64;
    let compressed_size = compressed.len() as u64;
    let name_len = MEMBER_NAME.len() as u16;

    // The single entry switches to zip64 when either size cannot be stored in a
    // 32-bit field; both sizes then move into the zip64 extra together. The
    // entry's own local-header offset is always 0 (it is the first member), so
    // it never needs zip64 on its own account.
    let entry_zip64 = uncompressed_size >= zip64_at || compressed_size >= zip64_at;
    let version_needed = if entry_zip64 {
        ZIP64_VERSION
    } else {
        BASE_VERSION
    };
    let (stored_csize, stored_usize) = if entry_zip64 {
        (ZIP64_SENTINEL, ZIP64_SENTINEL)
    } else {
        (compressed_size as u32, uncompressed_size as u32)
    };
    // The local and central headers share one zip64 extra: original size then
    // compressed size, each 8 bytes (header id 0x0001). Empty when not zip64.
    let extra = if entry_zip64 {
        zip64_size_extra(uncompressed_size, compressed_size)
    } else {
        Vec::new()
    };
    let extra_len = extra.len() as u16;

    let mut out = Vec::new();

    // Local file header.
    out.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // signature
    out.extend_from_slice(&version_needed.to_le_bytes()); // version needed
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&8u16.to_le_bytes()); // method: deflate
    out.extend_from_slice(&DOS_TIME.to_le_bytes()); // mod time
    out.extend_from_slice(&DOS_DATE.to_le_bytes()); // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&stored_csize.to_le_bytes());
    out.extend_from_slice(&stored_usize.to_le_bytes());
    out.extend_from_slice(&name_len.to_le_bytes());
    out.extend_from_slice(&extra_len.to_le_bytes()); // extra length
    out.extend_from_slice(MEMBER_NAME);
    out.extend_from_slice(&extra);
    out.extend_from_slice(&compressed);

    // Central directory header.
    let cd_offset = out.len() as u64;
    out.extend_from_slice(&0x0201_4b50u32.to_le_bytes()); // signature
    out.extend_from_slice(&version_needed.to_le_bytes()); // version made by
    out.extend_from_slice(&version_needed.to_le_bytes()); // version needed
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&8u16.to_le_bytes()); // method
    out.extend_from_slice(&DOS_TIME.to_le_bytes()); // mod time
    out.extend_from_slice(&DOS_DATE.to_le_bytes()); // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&stored_csize.to_le_bytes());
    out.extend_from_slice(&stored_usize.to_le_bytes());
    out.extend_from_slice(&name_len.to_le_bytes());
    out.extend_from_slice(&extra_len.to_le_bytes()); // extra length
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length
    out.extend_from_slice(&0u16.to_le_bytes()); // disk number start
    out.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
    out.extend_from_slice(&0u32.to_le_bytes()); // external attributes
    out.extend_from_slice(&0u32.to_le_bytes()); // local header offset
    out.extend_from_slice(MEMBER_NAME);
    out.extend_from_slice(&extra);
    let cd_size = out.len() as u64 - cd_offset;

    // Zip64 end-of-central-directory record and locator, emitted whenever a
    // field the classic end record stores in 32 bits would overflow.
    if entry_zip64 || cd_offset >= zip64_at || cd_size >= zip64_at {
        let zip64_eocd_offset = out.len() as u64;
        out.extend_from_slice(&0x0606_4b50u32.to_le_bytes()); // signature
        out.extend_from_slice(&44u64.to_le_bytes()); // size of remainder (record - 12)
        out.extend_from_slice(&ZIP64_VERSION.to_le_bytes()); // version made by
        out.extend_from_slice(&ZIP64_VERSION.to_le_bytes()); // version needed
        out.extend_from_slice(&0u32.to_le_bytes()); // number of this disk
        out.extend_from_slice(&0u32.to_le_bytes()); // disk with start of CD
        out.extend_from_slice(&1u64.to_le_bytes()); // entries on this disk
        out.extend_from_slice(&1u64.to_le_bytes()); // total entries
        out.extend_from_slice(&cd_size.to_le_bytes()); // size of CD
        out.extend_from_slice(&cd_offset.to_le_bytes()); // offset of CD

        out.extend_from_slice(&0x0706_4b50u32.to_le_bytes()); // locator signature
        out.extend_from_slice(&0u32.to_le_bytes()); // disk with zip64 EOCD
        out.extend_from_slice(&zip64_eocd_offset.to_le_bytes()); // offset of zip64 EOCD
        out.extend_from_slice(&1u32.to_le_bytes()); // total number of disks
    }

    // End of central directory. Overflowing fields carry the zip64 sentinel and
    // their real values live in the zip64 record written just above.
    let eocd_cd_size = if cd_size >= zip64_at {
        ZIP64_SENTINEL
    } else {
        cd_size as u32
    };
    let eocd_cd_offset = if cd_offset >= zip64_at {
        ZIP64_SENTINEL
    } else {
        cd_offset as u32
    };
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // signature
    out.extend_from_slice(&0u16.to_le_bytes()); // this disk
    out.extend_from_slice(&0u16.to_le_bytes()); // cd start disk
    out.extend_from_slice(&1u16.to_le_bytes()); // entries this disk
    out.extend_from_slice(&1u16.to_le_bytes()); // total entries
    out.extend_from_slice(&eocd_cd_size.to_le_bytes());
    out.extend_from_slice(&eocd_cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment length

    out
}

/// A zip64 extended-information extra field (header id 0x0001) carrying the
/// 64-bit uncompressed then compressed size. Both sizes are always present
/// together, matching the sentinels written into the 32-bit size fields.
fn zip64_size_extra(uncompressed_size: u64, compressed_size: u64) -> Vec<u8> {
    let mut extra = Vec::with_capacity(20);
    extra.extend_from_slice(&0x0001u16.to_le_bytes()); // header id: zip64
    extra.extend_from_slice(&16u16.to_le_bytes()); // data size: two 8-byte values
    extra.extend_from_slice(&uncompressed_size.to_le_bytes());
    extra.extend_from_slice(&compressed_size.to_le_bytes());
    extra
}

#[cfg(test)]
mod tests {
    use super::{
        DOS_DATE, MEMBER_NAME, ZIP64_SENTINEL, ZIP64_VERSION, wrap_voxjz, wrap_voxjz_with,
    };
    use crate::unwrap_voxjz;

    const EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    const ZIP64_EOCD_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];
    const ZIP64_LOCATOR_SIG: [u8; 4] = [0x50, 0x4b, 0x06, 0x07];
    const ZIP64_EXTRA_ID: [u8; 2] = [0x01, 0x00];

    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([bytes[at], bytes[at + 1]])
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
    }

    #[test]
    fn classic_archive_round_trips_without_zip64() {
        let member = br#"{"version":1,"main":{}}"#;
        let bytes = wrap_voxjz(member);

        // The common path stays classic: a single 22-byte end record, with no
        // zip64 locator preceding it.
        let eocd = bytes.len() - 22;
        assert_eq!(bytes[eocd..eocd + 4], EOCD_SIG);
        assert_ne!(bytes[eocd - 20..eocd - 16], ZIP64_LOCATOR_SIG);
        assert_eq!(u16_at(&bytes, 28), 0); // local extra length: none
        assert_eq!(u16_at(&bytes, 12), DOS_DATE); // valid 1980-01-01 mod date

        assert_eq!(unwrap_voxjz(&bytes).unwrap(), member);
    }

    #[test]
    fn zip64_archive_round_trips_and_is_well_formed() {
        let member = br#"{"version":1,"main":{"note":"forced zip64"}}"#;
        // Force the zip64 path on a tiny payload by lowering the threshold.
        let bytes = wrap_voxjz_with(member, 1);

        // The local header advertises zip64 and sends both sizes to the
        // sentinel, with the real sizes in a zip64 extra field after the name.
        assert_eq!(u16_at(&bytes, 4), ZIP64_VERSION); // version needed
        assert_eq!(u32_at(&bytes, 18), ZIP64_SENTINEL); // compressed size
        assert_eq!(u32_at(&bytes, 22), ZIP64_SENTINEL); // uncompressed size
        let name_len = u16_at(&bytes, 26) as usize;
        let extra_len = u16_at(&bytes, 28) as usize;
        assert_eq!(name_len, MEMBER_NAME.len());
        let extra = &bytes[30 + name_len..30 + name_len + extra_len];
        assert_eq!(extra[0..2], ZIP64_EXTRA_ID);
        assert_eq!(u16_at(extra, 2), 16); // two 8-byte values

        // The zip64 end record and locator sit just before the classic end
        // record, which now defers its central-directory offset to the sentinel.
        let eocd = bytes.len() - 22;
        let locator = eocd - 20;
        let zip64_eocd = locator - 56;
        assert_eq!(bytes[eocd..eocd + 4], EOCD_SIG);
        assert_eq!(bytes[locator..locator + 4], ZIP64_LOCATOR_SIG);
        assert_eq!(bytes[zip64_eocd..zip64_eocd + 4], ZIP64_EOCD_SIG);
        assert_eq!(u32_at(&bytes, eocd + 16), ZIP64_SENTINEL); // offset of CD

        assert_eq!(unwrap_voxjz(&bytes).unwrap(), member);
    }
}
