use crate::{GridResolution, PositiveF64};
use clap::{ArgGroup, Args};

/// The `voxelize` grid-resolution controls. Flattened onto the command,
/// which requires exactly one of the two flags.
#[derive(Clone, Debug, Args)]
#[command(group(
    ArgGroup::new("grid_resolution").required(true).args(["resolution", "voxel_size"])
))]
pub struct GridResolutionOptions {
    /// Grid resolution in voxels along the longest axis; other axes preserve
    /// aspect.
    #[arg(value_name = "resolution", long, value_parser = clap::value_parser!(u32).range(1..))]
    resolution: Option<u32>,

    /// Edge length of one voxel in meters, keeping the mesh's real-world size.
    #[arg(value_name = "voxel-size", long)]
    voxel_size: Option<PositiveF64>,
}

impl GridResolutionOptions {
    /// Resolves whichever flag is set into a [`GridResolution`]. The required
    /// `ArgGroup` guarantees exactly one is set.
    pub fn resolve(&self) -> GridResolution {
        self.resolution
            .map(GridResolution::VoxelGridLength)
            .or_else(|| {
                self.voxel_size
                    .map(|size| GridResolution::MetersPerVoxel(size.0))
            })
            .expect("the grid_resolution ArgGroup requires one flag")
    }
}

#[cfg(test)]
mod tests {
    use crate::{GridResolution, GridResolutionOptions};
    use clap::Parser;

    /// A throwaway command flattening the grid-resolution options, so their flags
    /// parse as they do on `voxelize`.
    #[derive(Parser)]
    struct Harness {
        #[command(flatten)]
        options: GridResolutionOptions,
    }

    /// The resolution `args` resolve to. Panics if they fail to parse.
    fn resolve(args: &[&str]) -> GridResolution {
        let mut argv = vec!["test"];
        argv.extend_from_slice(args);
        Harness::try_parse_from(argv).unwrap().options.resolve()
    }

    /// Whether `args` fail to parse.
    fn rejects(args: &[&str]) -> bool {
        let mut argv = vec!["test"];
        argv.extend_from_slice(args);
        Harness::try_parse_from(argv).is_err()
    }

    #[test]
    fn resolution_resolves_to_a_count() {
        assert!(matches!(
            resolve(&["--resolution", "32"]),
            GridResolution::VoxelGridLength(32)
        ));
    }

    #[test]
    fn voxel_size_resolves_to_a_size() {
        let resolution = resolve(&["--voxel-size", "0.25"]);
        assert!(matches!(resolution, GridResolution::MetersPerVoxel(size) if size == 0.25));
    }

    #[test]
    fn a_zero_count_is_rejected() {
        assert!(rejects(&["--resolution", "0"]));
    }

    #[test]
    fn a_non_positive_or_nan_size_is_rejected() {
        assert!(rejects(&["--voxel-size", "0"]));
        assert!(rejects(&["--voxel-size", "-1"]));
        assert!(rejects(&["--voxel-size", "nan"]));
    }

    #[test]
    fn the_two_flags_are_mutually_exclusive() {
        assert!(rejects(&["--resolution", "32", "--voxel-size", "0.25"]));
    }

    #[test]
    fn one_flag_is_required() {
        assert!(rejects(&[]));
    }
}
