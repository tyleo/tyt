use std::mem;

/// Sorts `keys` by their upper halves, least first, with `spare`'s room.
/// Keys whose upper halves match keep their order. This radix sort beats
/// `sort_unstable` on the keys a moving head sorts.
pub fn sort_keys(keys: &mut Vec<u64>, spare: &mut Vec<u64>) {
    spare.clear();
    spare.resize(keys.len(), 0);

    // Three passes of 11 bits cover an upper half.
    for shift in [32, 43, 54] {
        let digit = |key: u64| (key >> shift) as usize & 0x7ff;
        let mut starts = [0; 0x800];

        for &key in keys.iter() {
            starts[digit(key)] += 1;
        }

        let mut next = 0;

        for start in &mut starts {
            next += mem::replace(start, next);
        }

        for &key in keys.iter() {
            spare[starts[digit(key)]] = key;
            starts[digit(key)] += 1;
        }

        mem::swap(keys, spare);
    }
}

#[cfg(test)]
mod tests {
    use crate::sort_keys;

    #[test]
    fn keys_sort_by_their_upper_halves_and_ties_keep_their_order() {
        // Upper halves that differ within each pass's bits.
        let uppers = [0x7f80_0000, 3, 0x0040_0001, 3, 0, 0x0000_0801, 0x0040_0000];

        let mut keys: Vec<u64> = uppers
            .iter()
            .zip(0..)
            .map(|(upper, index)| upper << 32 | index)
            .collect();

        let mut sorted = keys.clone();

        sorted.sort_unstable();
        sort_keys(&mut keys, &mut Vec::new());

        assert_eq!(keys, sorted);
    }
}
