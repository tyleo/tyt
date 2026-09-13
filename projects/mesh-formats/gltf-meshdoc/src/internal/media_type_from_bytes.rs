use meshdoc::MeshImageMediaType;

/// The media type whose signature `bytes` start with, or `None` for neither.
pub fn media_type_from_bytes(bytes: &[u8]) -> Option<MeshImageMediaType> {
    [MeshImageMediaType::Png, MeshImageMediaType::Jpeg]
        .into_iter()
        .find(|media_type| media_type.matches(bytes))
}
