/// The reflected IEEE CRC-32 polynomial.
const POLYNOMIAL: u32 = 0xEDB8_8320;

/// The CRC of each byte value.
const TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut byte = 0;
    while byte < 256 {
        let mut crc = byte as u32;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 1 == 1 {
                POLYNOMIAL ^ (crc >> 1)
            } else {
                crc >> 1
            };
            bit += 1;
        }
        table[byte] = crc;
        byte += 1;
    }
    table
};

/// The IEEE CRC-32 of `bytes`, the checksum a zip archive stores for its
/// member.
pub fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(!0u32, |crc, &byte| {
        TABLE[usize::from(crc as u8 ^ byte)] ^ (crc >> 8)
    })
}

#[cfg(test)]
mod tests {
    use crate::crc32;

    /// The standard check value of the IEEE CRC-32.
    #[test]
    fn matches_the_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
    }
}
