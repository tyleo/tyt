use crate::{VoxjVoxExt, VoxjVoxMain};
use voxcore::VoxMain;

/// Gives a bare main an empty `ext` block, the [`VoxjVoxMain`]
/// [`to_voxj_file`](crate::to_voxj_file) writes with no block.
pub fn to_voxj_vox_main(main: VoxMain<()>) -> VoxjVoxMain {
    main.put_ext(VoxjVoxExt::default())
}

#[cfg(test)]
mod tests {
    use crate::{VoxjVoxExt, VoxjWriteOptions, to_voxj_file, to_voxj_vox_main};
    use voxcore::VoxMain;
    use voxj_codec::DependenciesImpl;

    #[test]
    fn puts_an_empty_ext_that_writes_no_block() {
        let main = to_voxj_vox_main(VoxMain::default());

        assert_eq!(main.ext(), &VoxjVoxExt::default());

        let file = to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap();

        assert_eq!(file.main.ext, None);
    }
}
