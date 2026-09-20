/// Each occurrence of a repeatable flag taking `N` tokens, which clap's
/// `num_args` guarantees `values` holds whole.
pub(crate) fn flag_occurrences<const N: usize>(
    values: &[String],
) -> impl Iterator<Item = &[String; N]> {
    let (occurrences, remainder) = values.as_chunks::<N>();

    assert!(
        remainder.is_empty(),
        "clap's num_args guarantees whole occurrences"
    );

    occurrences.iter()
}
