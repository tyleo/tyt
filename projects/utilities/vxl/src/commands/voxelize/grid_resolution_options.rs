use crate::{CliValue, Error, PositiveF64, Result};
use clap::{ArgGroup, Args};
use voxsmith::{GridResolution, ResolutionAxis};

/// The `voxelize` grid-resolution controls. Flattened onto the command, which
/// takes at most one of the two flags and defaults to one voxel per meter when
/// neither is given.
#[derive(Clone, Debug, Args)]
#[command(group(
    ArgGroup::new("grid_resolution").args(["resolution", "voxel_size"])
))]
pub struct GridResolutionOptions {
    /// Voxel count `n` along a chosen axis, the other axes preserving aspect.
    /// `<axis>` is one of:
    ///
    /// 1. `long`: the mesh's longest extent.
    /// 2. `short`: the mesh's shortest extent.
    /// 3. `x` | `y` | `z`: that specific axis.
    #[arg(value_names = ["axis", "n"], long, num_args = 2, verbatim_doc_comment)]
    resolution: Option<Vec<String>>,

    /// Edge length of one voxel in meters, keeping the mesh's real-world size.
    #[arg(value_name = "voxel-size", long)]
    voxel_size: Option<PositiveF64>,
}

impl GridResolutionOptions {
    /// Resolves the grid-resolution flags into a [`GridResolution`]. `--resolution`
    /// pins one axis to a voxel count; `--voxel-size` sets a real-world voxel size;
    /// with neither it defaults to one voxel per meter. Rejects a `--resolution`
    /// with an unknown axis or a count below one.
    pub fn resolve(&self) -> Result<GridResolution> {
        if let Some(values) = &self.resolution {
            // `num_args = 2` guarantees exactly two values when the flag is set.
            let [axis, count] = values.as_slice() else {
                return Err(Error::usage("--resolution takes an axis and a count"));
            };

            let axis = ResolutionAxis::parse(axis)
                .map_err(|message| Error::usage(format!("--resolution axis: {message}")))?;

            let count = match count.parse::<u32>() {
                Ok(count) if count >= 1 => count,
                _ => {
                    return Err(Error::usage(format!(
                        "--resolution count `{count}` must be a whole number of at least 1"
                    )));
                }
            };

            return Ok(GridResolution::AxisVoxelCount { axis, count });
        }

        if let Some(size) = self.voxel_size {
            return Ok(GridResolution::MetersPerVoxel(size.0));
        }

        Ok(GridResolution::MetersPerVoxel(1.0))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::GridResolutionOptions;
    use clap::Parser;
    use voxsmith::{GridResolution, ResolutionAxis};

    /// A throwaway command flattening the grid-resolution options, so their flags
    /// parse as they do on `voxelize`.
    #[derive(Parser)]
    struct Harness {
        #[command(flatten)]
        options: GridResolutionOptions,
    }

    /// The resolution `args` resolve to. Panics if they fail to parse or resolve.
    fn resolve(args: &[&str]) -> GridResolution {
        let mut argv = vec!["test"];
        argv.extend_from_slice(args);
        Harness::try_parse_from(argv)
            .unwrap()
            .options
            .resolve()
            .unwrap()
    }

    /// Whether `args` fail to parse or resolve.
    fn rejects(args: &[&str]) -> bool {
        let mut argv = vec!["test"];
        argv.extend_from_slice(args);
        match Harness::try_parse_from(argv) {
            Ok(harness) => harness.options.resolve().is_err(),
            Err(_) => true,
        }
    }

    #[test]
    fn resolution_resolves_to_an_axis_count() {
        assert!(matches!(
            resolve(&["--resolution", "long", "32"]),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Long,
                count: 32
            }
        ));
    }

    #[test]
    fn resolution_accepts_the_short_axis() {
        assert!(matches!(
            resolve(&["--resolution", "short", "16"]),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Short,
                count: 16
            }
        ));
    }

    #[test]
    fn resolution_accepts_a_named_axis() {
        assert!(matches!(
            resolve(&["--resolution", "y", "8"]),
            GridResolution::AxisVoxelCount {
                axis: ResolutionAxis::Y,
                count: 8
            }
        ));
    }

    #[test]
    fn voxel_size_resolves_to_a_size() {
        let resolution = resolve(&["--voxel-size", "0.25"]);
        assert!(matches!(resolution, GridResolution::MetersPerVoxel(size) if size == 0.25));
    }

    #[test]
    fn neither_flag_defaults_to_one_meter_per_voxel() {
        assert!(matches!(resolve(&[]), GridResolution::MetersPerVoxel(size) if size == 1.0));
    }

    #[test]
    fn a_zero_count_is_rejected() {
        assert!(rejects(&["--resolution", "long", "0"]));
    }

    #[test]
    fn an_unknown_axis_is_rejected() {
        assert!(rejects(&["--resolution", "diagonal", "4"]));
    }

    #[test]
    fn a_lone_axis_without_a_count_is_rejected() {
        assert!(rejects(&["--resolution", "long"]));
    }

    #[test]
    fn a_non_positive_or_nan_size_is_rejected() {
        assert!(rejects(&["--voxel-size", "0"]));
        assert!(rejects(&["--voxel-size", "-1"]));
        assert!(rejects(&["--voxel-size", "nan"]));
    }

    #[test]
    fn the_two_flags_are_mutually_exclusive() {
        assert!(rejects(&[
            "--resolution",
            "long",
            "32",
            "--voxel-size",
            "0.25"
        ]));
    }
}
