/// `count` as a word a shader reads. Panics past `u32::MAX`, which no
/// scene a device holds reaches.
pub fn to_word(count: usize) -> u32 {
    u32::try_from(count).expect("a scene's counts fit a word")
}
