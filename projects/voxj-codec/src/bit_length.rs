/// Number of binary digits in `n`, with `bit_length(0) == 0`.
pub fn bit_length(mut n: u32) -> u32 {
    let mut len = 0;
    while n > 0 {
        n /= 2;
        len += 1;
    }
    len
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
