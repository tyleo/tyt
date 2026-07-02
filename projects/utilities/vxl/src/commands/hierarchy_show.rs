use crate::{BoundsView, Dependencies, Error, Format, PatternView, Result, TransformView};
use clap::Parser;
use std::{
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

/// Prints the scene graph as a tree with box-drawing glyphs, marking instanced
/// nodes and listing unplaced nodes and orphan objects.
#[derive(Clone, Debug, Parser)]
#[command(name = "show")]
pub struct HierarchyShow {
    /// The input voxel file, in any supported format.
    #[arg(value_name = "input")]
    input: PathBuf,

    /// Gitignore-style patterns matched against node and object paths. When set,
    /// only matched nodes and objects and their ancestors print. A plain pattern
    /// selects, a leading `!` deselects, a trailing `/` matches nodes only, and
    /// the last matching pattern wins. Repeat to pass several.
    #[arg(value_name = "pattern")]
    patterns: Vec<String>,

    /// Source format of the input. Inferred from its extension when omitted.
    #[arg(value_name = "from", long)]
    from: Option<Format>,

    /// Collapse repeat instances: expand a shared node's first placement and
    /// print each later placement as a non-expanded stub.
    #[arg(value_name = "collapse-instances", long = "collapse-instances")]
    collapse_instances: bool,

    /// Hide the ancestor chain above each match behind an `ancestors` marker.
    /// Requires a `pattern`.
    #[arg(
        value_name = "collapse-ancestors",
        long = "collapse-ancestors",
        requires = "patterns"
    )]
    collapse_ancestors: bool,

    /// Hide the descendants of each match behind a `descendants` marker.
    /// Requires a `pattern`.
    #[arg(
        value_name = "collapse-descendants",
        long = "collapse-descendants",
        requires = "patterns"
    )]
    collapse_descendants: bool,

    /// Prepend each node's transform as a nested subtree. Up to three positional
    /// values `[space] [rot-unit] [precision]`: `space` is `local` (default) or
    /// `world`, `rot-unit` is `rad` (default) or `deg`, `precision` is the
    /// decimal places (default `2`).
    #[arg(
        value_names = ["space", "rot-unit", "precision"],
        long = "show-transforms",
        num_args = 0..=3,
    )]
    show_transforms: Option<Vec<String>>,

    /// Append each object's grid bounds as a nested subtree. Up to two positional
    /// values `[space] [precision]`: `space` is `local` (default) or `world`,
    /// `precision` is the decimal places (default `2`).
    #[arg(
        value_names = ["space", "precision"],
        long = "show-bounds",
        num_args = 0..=2,
    )]
    show_bounds: Option<Vec<String>>,

    /// Append each object's extents (`max - min`) as a nested subtree, with the
    /// same `[space] [precision]` arguments as `--show-bounds`.
    #[arg(
        value_names = ["space", "precision"],
        long = "show-extents",
        num_args = 0..=2,
    )]
    show_extents: Option<Vec<String>>,

    /// Append each object's referenced palettes as a nested subtree, one child
    /// per palette showing its index and cell count.
    #[arg(value_name = "show-palettes", long = "show-palettes")]
    show_palettes: bool,
}

impl HierarchyShow {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let transforms = self
            .show_transforms
            .as_deref()
            .map(parse_transform_view)
            .transpose()?;

        let bounds = self
            .show_bounds
            .as_deref()
            .map(parse_bounds_view)
            .transpose()?;

        let extents = self
            .show_extents
            .as_deref()
            .map(parse_bounds_view)
            .transpose()?;

        // The collapse flags require a pattern, so clap guarantees they are false
        // when none is given; bundling them with the globs keeps that invariant.
        let pattern = (!self.patterns.is_empty()).then_some(PatternView {
            globs: self.patterns,
            collapse_ancestors: self.collapse_ancestors,
            collapse_descendants: self.collapse_descendants,
        });

        dependencies.hierarchy_show(
            &self.input,
            self.from,
            pattern,
            self.collapse_instances,
            transforms,
            bounds,
            extents,
            self.show_palettes,
        )
    }
}

/// Parses `[space] [rot-unit] [precision]` into a [`TransformView`].
fn parse_transform_view(values: &[String]) -> Result<TransformView> {
    let world = parse_space(values.first())?;
    let degrees = parse_rot_unit(values.get(1))?;
    let precision = parse_precision(values.get(2))?;

    Ok(TransformView {
        world,
        degrees,
        precision,
    })
}

/// Parses `[space] [precision]` into a [`BoundsView`].
fn parse_bounds_view(values: &[String]) -> Result<BoundsView> {
    let world = parse_space(values.first())?;
    let precision = parse_precision(values.get(1))?;

    Ok(BoundsView { world, precision })
}

/// Parses the `space` value: `local` (default) is false, `world` is true.
fn parse_space(value: Option<&String>) -> Result<bool> {
    match value.map(String::as_str) {
        None | Some("local") => Ok(false),

        Some("world") => Ok(true),

        Some(other) => Err(invalid(format!(
            "space must be 'local' or 'world', got '{other}'"
        ))),
    }
}

/// Parses the `rot-unit` value: `rad` (default) is false, `deg` is true.
fn parse_rot_unit(value: Option<&String>) -> Result<bool> {
    match value.map(String::as_str) {
        None | Some("rad") => Ok(false),

        Some("deg") => Ok(true),

        Some(other) => Err(invalid(format!(
            "rot-unit must be 'rad' or 'deg', got '{other}'"
        ))),
    }
}

/// Parses the `precision` value, a non-negative integer, defaulting to `2`.
fn parse_precision(value: Option<&String>) -> Result<usize> {
    match value {
        None => Ok(2),

        Some(text) => text
            .parse::<usize>()
            .map_err(|error| invalid(format!("precision must be a non-negative integer: {error}"))),
    }
}

/// A parse error as an invalid-input [`Error`].
fn invalid(message: String) -> Error {
    Error::IO(IOError::new(ErrorKind::InvalidInput, message))
}

#[cfg(test)]
mod tests {
    use crate::commands::{
        HierarchyShow,
        hierarchy_show::{parse_bounds_view, parse_transform_view},
    };
    use clap::Parser;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn a_collapse_flag_requires_a_pattern() {
        // Without a pattern, clap rejects the collapse flag rather than silently
        // ignoring it.
        assert!(
            HierarchyShow::try_parse_from(["show", "in.voxj", "--collapse-ancestors"]).is_err()
        );
        // With a pattern it parses.
        assert!(
            HierarchyShow::try_parse_from(["show", "in.voxj", "door", "--collapse-ancestors"])
                .is_ok()
        );
    }

    #[test]
    fn transform_view_defaults_to_local_radians_precision_two() {
        let view = parse_transform_view(&[]).unwrap();
        assert!(!view.world && !view.degrees && view.precision == 2);
    }

    #[test]
    fn transform_view_parses_all_three_values() {
        let view = parse_transform_view(&strings(&["world", "deg", "4"])).unwrap();
        assert!(view.world && view.degrees && view.precision == 4);
    }

    #[test]
    fn bounds_view_parses_space_and_precision() {
        let view = parse_bounds_view(&strings(&["world", "3"])).unwrap();
        assert!(view.world && view.precision == 3);
        let default = parse_bounds_view(&[]).unwrap();
        assert!(!default.world && default.precision == 2);
    }

    #[test]
    fn a_bad_space_or_unit_is_an_error() {
        assert!(parse_transform_view(&strings(&["sideways"])).is_err());
        assert!(parse_transform_view(&strings(&["local", "turns"])).is_err());
        assert!(parse_bounds_view(&strings(&["local", "-1"])).is_err());
    }
}
