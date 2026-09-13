/// `values` as little-endian bytes, the form a buffer holds floats in.
pub fn f32_bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}
