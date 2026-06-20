/// Number of binary digits in `n`, with `bit_length(0) == 0`.
pub fn bit_length(n: u32) -> u32 {
    u32::BITS - n.leading_zeros()
}

#[cfg(test)]
mod tests {
    use crate::bit_length;

    #[test]
    fn matches_reference() {
        assert_eq!(bit_length(0), 0);
        assert_eq!(bit_length(1), 1);
        assert_eq!(bit_length(2), 2);
        assert_eq!(bit_length(255), 8);
    }
}
