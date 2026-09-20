use vox_value_language::{Components, Dimension, Domain, Value};

/// Each entry's index in `domain`, a `u32` vec1 array of `count` entries
/// counting up from `0`.
pub(crate) fn compute_index(domain: Domain, count: usize) -> Value {
    Value::new(
        domain,
        Dimension::Vec1,
        Components::U32((0..count).map(|index| index as u32).collect()),
    )
    .expect("one component per entry fills a vec1 array")
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::compute_index;
    use vox_value_language::{Components, Domain};

    #[test]
    fn the_index_counts_the_entries_up() {
        let value = compute_index(Domain::Face, 3);

        assert_eq!(value.domain(), Domain::Face);
        assert_eq!(value.components(), &Components::U32(vec![0, 1, 2]));
    }
}
