use crate::{
    Result,
    ext::{GoxlVoxMain, to_goxl_file_with_ext},
};
use goxl_codec::{EncodePng, to_gox_file_bytes};

/// Writes a [`GoxlVoxMain`] and its ext to the bytes of a Goxel `.gox` file
/// through `dependencies`, the bytes form of [`to_goxl_file_with_ext`] and
/// the inverse of
/// [`from_goxl_bytes_with_ext`](crate::codec::from_goxl_bytes_with_ext).
pub fn to_goxl_bytes_with_ext<D: EncodePng>(
    dependencies: &D,
    state: &GoxlVoxMain,
) -> Result<Vec<u8>> {
    let file = to_goxl_file_with_ext(state)?;

    Ok(to_gox_file_bytes(dependencies, &file))
}

#[cfg(test)]
mod tests {
    use crate::{
        codec::{from_goxl_bytes_with_ext, to_goxl_bytes_with_ext},
        ext::GoxlVoxMain,
    };
    use goxl_codec::DependenciesImpl;

    /// A state with no ext writes a synthesized file that loads back carrying
    /// the ext the typed path keeps.
    #[test]
    fn round_trips_the_ext_through_bytes() {
        let bytes = to_goxl_bytes_with_ext(&DependenciesImpl, &GoxlVoxMain::default()).unwrap();
        assert!(bytes.starts_with(b"GOX "));

        let reloaded = from_goxl_bytes_with_ext(&DependenciesImpl, &bytes).unwrap();
        assert!(reloaded.ext().is_some(), "a loaded file carries its ext");
    }
}
