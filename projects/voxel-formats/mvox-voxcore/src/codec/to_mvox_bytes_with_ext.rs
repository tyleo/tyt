use crate::{
    Result,
    ext::{MVoxVoxMain, to_mvox_file_with_ext},
};
use mvox_codec::to_mvox_file_bytes;

/// Writes a [`MVoxVoxMain`] and its ext to the bytes of a MagicaVoxel `.vox`
/// file, the bytes form of [`to_mvox_file_with_ext`] and the inverse of
/// [`from_mvox_bytes_with_ext`](crate::codec::from_mvox_bytes_with_ext).
pub fn to_mvox_bytes_with_ext(state: &MVoxVoxMain) -> Result<Vec<u8>> {
    let file = to_mvox_file_with_ext(state)?;

    Ok(to_mvox_file_bytes(&file))
}

#[cfg(test)]
mod tests {
    use crate::{
        codec::{from_mvox_bytes_with_ext, to_mvox_bytes_with_ext},
        ext::MVoxVoxMain,
    };

    /// A state with no ext writes a synthesized file that loads back carrying
    /// the ext the typed path keeps.
    #[test]
    fn round_trips_the_ext_through_bytes() {
        let bytes = to_mvox_bytes_with_ext(&MVoxVoxMain::default()).unwrap();
        assert!(bytes.starts_with(b"VOX "));

        let reloaded = from_mvox_bytes_with_ext(&bytes).unwrap();
        assert!(reloaded.ext().is_some(), "a loaded file carries its ext");
    }
}
