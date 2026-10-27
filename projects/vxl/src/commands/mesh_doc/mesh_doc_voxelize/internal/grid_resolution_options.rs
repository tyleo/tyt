use crate::{CliValue, Error, PositiveF64, Result};
use clap::{ArgGroup, Args};
use voxsmith::utilities::{GridResolution, ResolutionReference};

/// The voxelize commands' voxel-size controls. Flattened onto each command,
/// which takes at most one of the two flags.
#[derive(Clone, Debug, Args)]
#[command(group(
    ArgGroup::new("grid_resolution").args(["resolution", "voxel_size"])
))]
pub struct GridResolutionOptions {
    /// Voxel count `n` along a reference side. The voxel size is that side divided by `n`. World references measure the whole model. Object references take the longest or shortest object. `reference` is one of:
    ///
    /// - `longest-world` or `shortest-world`
    /// - `world-x`, `world-y`, or `world-z`
    /// - `longest-object` or `shortest-object`
    /// - `longest-object-x`, `longest-object-y`, or `longest-object-z`
    /// - `shortest-object-x`, `shortest-object-y`, or `shortest-object-z`
    #[arg(value_names = ["reference", "n"], long, num_args = 2, verbatim_doc_comment)]
    resolution: Option<Vec<String>>,

    /// Edge length of one voxel in meters.
    #[arg(value_name = "voxel-size", long)]
    voxel_size: Option<PositiveF64>,
}

impl GridResolutionOptions {
    /// Resolves the voxel-size flags into a [`GridResolution`], `None` when
    /// neither is given. Rejects an unknown reference or a count below one.
    pub fn resolve(&self) -> Result<Option<GridResolution>> {
        if let Some(values) = &self.resolution {
            // `num_args = 2` guarantees exactly two values when the flag is set.
            let [reference, count] = values.as_slice() else {
                return Err(Error::usage("--resolution takes a reference and a count"));
            };

            let reference = ResolutionReference::parse(reference)
                .map_err(|message| Error::usage(format!("--resolution reference: {message}")))?;

            let count = match count.parse::<u32>() {
                Ok(count) if count >= 1 => count,

                _ => {
                    return Err(Error::usage(format!(
                        "--resolution count `{count}` must be a whole number of at least 1"
                    )));
                }
            };

            return Ok(Some(GridResolution::ReferenceCount { reference, count }));
        }

        Ok(self
            .voxel_size
            .map(|size| GridResolution::VoxelSize(size.0)))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::GridResolutionOptions;
    use clap::Parser;
    use voxsmith::utilities::{GridResolution, ResolutionReference};

    /// A throwaway command flattening the grid-resolution options, so their flags
    /// parse as they do on `mesh-doc voxelize`.
    #[derive(Parser)]
    struct Harness {
        #[command(flatten)]
        options: GridResolutionOptions,
    }

    /// The resolution `args` resolve to. Panics if they fail to parse or resolve.
    fn resolve(args: &[&str]) -> Option<GridResolution> {
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
    fn resolution_resolves_to_a_reference_count() {
        assert_eq!(
            resolve(&["--resolution", "longest-world", "32"]),
            Some(GridResolution::ReferenceCount {
                reference: ResolutionReference::LongestWorld,
                count: 32
            })
        );
        assert_eq!(
            resolve(&["--resolution", "shortest-world", "16"]),
            Some(GridResolution::ReferenceCount {
                reference: ResolutionReference::ShortestWorld,
                count: 16
            })
        );
        assert_eq!(
            resolve(&["--resolution", "world-y", "8"]),
            Some(GridResolution::ReferenceCount {
                reference: ResolutionReference::WorldY,
                count: 8
            })
        );
    }

    #[test]
    fn voxel_size_resolves_to_a_size() {
        assert_eq!(
            resolve(&["--voxel-size", "0.25"]),
            Some(GridResolution::VoxelSize(0.25))
        );
    }

    #[test]
    fn neither_flag_resolves_to_none() {
        assert_eq!(resolve(&[]), None);
    }

    #[test]
    fn a_zero_count_is_rejected() {
        assert!(rejects(&["--resolution", "longest-world", "0"]));
    }

    #[test]
    fn an_unknown_reference_is_rejected() {
        assert!(rejects(&["--resolution", "long", "4"]));
    }

    #[test]
    fn a_lone_reference_without_a_count_is_rejected() {
        assert!(rejects(&["--resolution", "longest-world"]));
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
            "longest-world",
            "32",
            "--voxel-size",
            "0.25"
        ]));
    }
}
