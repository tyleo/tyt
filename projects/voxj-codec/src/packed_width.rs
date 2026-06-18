use crate::bit_length;

/// Bit-packing width for a palette with `cell_count` cells.
pub fn packed_width(cell_count: usize) -> u32 {
    bit_length((cell_count.max(1) - 1) as u32).max(1)
}

#[cfg(test)]
mod tests {
    use crate::packed_width;

    #[test]
    fn matches_reference() {
        assert_eq!(packed_width(2), 1);
        assert_eq!(packed_width(256), 8);
    }
}
