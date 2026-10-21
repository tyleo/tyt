/// Appends `values` and a zero as one `vec4<i32>`.
pub fn push_i32s(bytes: &mut Vec<u8>, values: [i32; 3]) {
    for value in values.into_iter().chain([0]) {
        bytes.extend(value.to_le_bytes());
    }
}
