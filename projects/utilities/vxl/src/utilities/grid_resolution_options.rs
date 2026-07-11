use crate::{GridResolution, Result};
use clap::{ArgGroup, Args};
use std::io::{Error as IOError, ErrorKind};

/// The `voxelize` grid-resolution controls. Flattened onto the command,
/// which requires exactly one of the two flags.
#[derive(Clone, Debug, Args)]
#[command(group(
    ArgGroup::new("grid_resolution").required(true).args(["resolution", "voxel_size"])
))]
pub struct GridResolutionOptions {
    /// Grid resolution in voxels along the longest axis; other axes preserve
    /// aspect.
    #[arg(value_name = "resolution", long)]
    resolution: Option<u32>,

    /// Edge length of one voxel in meters, keeping the mesh's real-world size.
    #[arg(value_name = "voxel-size", long)]
    voxel_size: Option<f64>,
}

impl GridResolutionOptions {
    /// Resolves whichever flag is set into a [`GridResolution`].
    pub fn resolve(&self) -> Result<GridResolution> {
        match (self.resolution, self.voxel_size) {
            (Some(0), _) => Err(usage("--resolution must be at least 1")),
            (Some(length), _) => Ok(GridResolution::VoxelGridLength(length)),
            (_, Some(meters)) if meters <= 0.0 || meters.is_nan() => {
                Err(usage("--voxel-size must be greater than 0"))
            }
            (_, Some(meters)) => Ok(GridResolution::MetersPerVoxel(meters)),
            (None, None) => Err(usage("set --resolution or --voxel-size")),
        }
    }
}

/// A usage error for a rule clap cannot express, exiting non-zero with a message.
fn usage(message: &str) -> crate::Error {
    IOError::new(ErrorKind::InvalidInput, message).into()
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

    /// The resolution `args` resolve to, or an error.
    fn resolve(args: &[&str]) -> crate::Result<GridResolution> {
        let mut argv = vec!["test"];
        argv.extend_from_slice(args);
        Harness::try_parse_from(argv).unwrap().options.resolve()
    }

    #[test]
    fn resolution_resolves_to_a_count() {
        assert!(matches!(
            resolve(&["--resolution", "32"]).unwrap(),
            GridResolution::VoxelGridLength(32)
        ));
    }

    #[test]
    fn voxel_size_resolves_to_a_size() {
        let resolution = resolve(&["--voxel-size", "0.25"]).unwrap();
        assert!(matches!(resolution, GridResolution::MetersPerVoxel(size) if size == 0.25));
    }

    #[test]
    fn a_zero_count_is_rejected() {
        assert!(resolve(&["--resolution", "0"]).is_err());
    }

    #[test]
    fn a_non_positive_or_nan_size_is_rejected() {
        // A negative size is a clap parse error, so `resolve` only has to guard
        // the values that reach it: zero and NaN.
        assert!(resolve(&["--voxel-size", "0"]).is_err());
        assert!(resolve(&["--voxel-size", "nan"]).is_err());
    }

    #[test]
    fn the_two_flags_are_mutually_exclusive() {
        assert!(
            Harness::try_parse_from(["test", "--resolution", "32", "--voxel-size", "0.25"])
                .is_err()
        );
    }

    #[test]
    fn one_flag_is_required() {
        assert!(Harness::try_parse_from(["test"]).is_err());
    }
}
