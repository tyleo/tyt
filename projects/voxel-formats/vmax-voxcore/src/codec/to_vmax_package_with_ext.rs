use crate::{
    Result, VMaxWriteOptions,
    ext::{VMaxVoxMain, to_vmax_file_with_ext},
};
use vmax_codec::{
    CompressLzfse, EncodePng, EncodeVMaxPlist, EncodeVMaxSceneJson, Result as CodecResult,
    to_vmax_package as write_vmax_package,
};

/// Writes a [`VMaxVoxMain`] and its ext to a `.vmax` package through
/// `dependencies`, the package form of [`to_vmax_file_with_ext`] and the
/// inverse of
/// [`from_vmax_package_with_ext`](crate::codec::from_vmax_package_with_ext).
/// `options` and `write` work as on
/// [`to_vmax_package`](crate::codec::to_vmax_package).
pub fn to_vmax_package_with_ext<D, W>(
    dependencies: &D,
    state: &VMaxVoxMain,
    options: &VMaxWriteOptions,
    write: W,
) -> Result<()>
where
    D: CompressLzfse + EncodeVMaxPlist + EncodePng + EncodeVMaxSceneJson,
    W: FnMut(&str, &[u8]) -> CodecResult<()>,
{
    let file = to_vmax_file_with_ext(state, options)?;

    Ok(write_vmax_package(dependencies, &file, write)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        VMaxWriteOptions,
        codec::{from_vmax_package_with_ext, to_vmax_package_with_ext},
        ext::VMaxVoxMain,
    };
    use std::collections::HashMap;
    use vmax_codec::DependenciesImpl;

    /// A state with no ext writes a synthesized package that loads back
    /// carrying the ext the typed path keeps.
    #[test]
    fn round_trips_the_ext_through_an_in_memory_package() {
        let mut package: HashMap<String, Vec<u8>> = HashMap::new();
        to_vmax_package_with_ext(
            &DependenciesImpl,
            &VMaxVoxMain::default(),
            &VMaxWriteOptions::default(),
            |name, bytes| {
                package.insert(name.to_owned(), bytes.to_vec());
                Ok(())
            },
        )
        .unwrap();
        assert!(package.contains_key("scene.json"));

        let reloaded = from_vmax_package_with_ext(
            &DependenciesImpl,
            || Ok(package.keys().cloned().collect()),
            |name| Ok(package.get(name).cloned()),
        )
        .unwrap();
        assert!(reloaded.ext().is_some(), "a loaded package carries its ext");
    }
}
