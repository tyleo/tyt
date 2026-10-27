use crate::{Result, VoxjVoxMain, VoxjWriteOptions, to_voxj_file};
use voxj::{CostVoxjObject, EncodeBase64};
use voxj_codec::{EncodeVoxjJson, to_voxj_file_bytes};

/// Writes a [`VoxjVoxMain`] to compact `.voxj` JSON bytes, the bytes form of
/// [`to_voxj_file()`].
pub fn to_voxj_bytes<D: EncodeBase64 + CostVoxjObject + EncodeVoxjJson>(
    dependencies: &D,
    main: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<Vec<u8>> {
    let file = to_voxj_file(dependencies, main, options)?;
    Ok(to_voxj_file_bytes(dependencies, &file)?)
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, VoxjWriteOptions,
        codec::{to_voxj_bytes, to_voxj_pretty_bytes, to_voxjz_bytes},
        to_voxj_vox_main,
    };
    use voxcore::{VoxMain, VoxValue, VoxValuePool};
    use voxj_codec::DependenciesImpl;

    #[test]
    fn a_json_value_without_a_json_form_errors_on_write() {
        let mut main = VoxMain::default();

        main.retain_value_pool(VoxValuePool::json(vec![VoxValue::Number(f64::NAN)]));

        let main = to_voxj_vox_main(main);

        let options = VoxjWriteOptions::default();

        assert!(matches!(
            to_voxj_bytes(&DependenciesImpl, &main, &options),
            Err(Error::Codec(_))
        ));

        assert!(matches!(
            to_voxj_pretty_bytes(&DependenciesImpl, &main, &options),
            Err(Error::Codec(_))
        ));

        assert!(matches!(
            to_voxjz_bytes(&DependenciesImpl, &main, &options),
            Err(Error::Codec(_))
        ));
    }
}
