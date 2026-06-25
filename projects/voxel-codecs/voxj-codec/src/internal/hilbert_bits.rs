use crate::bit_length;

/// Hilbert `bits` per axis for the given `[X, Y, Z]` bounds: the bit width
/// needed to index the largest axis.
pub fn hilbert_bits(bounds: [u32; 3]) -> u32 {
    let max_dim = bounds[0].max(bounds[1]).max(bounds[2]);
    bit_length(max_dim.saturating_sub(1)).max(1)
}

#[cfg(test)]
mod tests {
    use crate::hilbert_bits;

    #[test]
    fn indexes_the_largest_axis() {
        assert_eq!(hilbert_bits([2, 2, 1]), 1);
    }
}
