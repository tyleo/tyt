/// Whether `bytes` start with the GLB magic.
pub fn is_glb(bytes: &[u8]) -> bool {
    bytes.starts_with(b"glTF")
}
