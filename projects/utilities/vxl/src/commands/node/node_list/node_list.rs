use crate::{Dependencies, Error, Result, VoxelInput, cli_value_parser, load_vox_main};
use clap::Parser;
use std::io::{Error as IOError, ErrorKind};
use voxcore::VoxMain;
use voxsmith::operations::node::{
    NodeListLayout, NodeListOptions, NodeListViews, OriginView, PatternView, TransformView,
    node_list,
};

/// Lists the scene graph's nodes as a box-glyph tree or as JSON records, marking
/// instanced nodes and listing unplaced nodes and orphan objects.
#[derive(Clone, Debug, Parser)]
#[command(name = "list")]
pub struct NodeList {
    #[command(flatten)]
    input: VoxelInput,

    /// Gitignore-style patterns matched against node and object paths. When
    /// given, only matched nodes and objects and their ancestors print. A plain
    /// pattern selects, a leading `!` deselects, a trailing `/` matches nodes
    /// only, and the last matching pattern wins. With none given, the whole
    /// scene graph prints.
    #[arg(value_name = "pattern")]
    patterns: Vec<String>,

    /// How to lay out the scene graph.
    #[arg(
        value_name = "layout",
        long,
        default_value = "box-hierarchy",
        value_parser = cli_value_parser::<NodeListLayout>()
    )]
    layout: NodeListLayout,

    /// Collapses repeat instances: expands a shared node's first placement and
    /// prints each later placement as a non-expanded stub.
    #[arg(value_name = "collapse-instances", long = "collapse-instances")]
    collapse_instances: bool,

    /// Hides the ancestor chain above each match behind an `ancestors` marker.
    /// Requires a `pattern`.
    #[arg(
        value_name = "collapse-ancestors",
        long = "collapse-ancestors",
        requires = "patterns"
    )]
    collapse_ancestors: bool,

    /// Hides the descendants of each match behind a `descendants` marker.
    /// Requires a `pattern`.
    #[arg(
        value_name = "collapse-descendants",
        long = "collapse-descendants",
        requires = "patterns"
    )]
    collapse_descendants: bool,

    /// Prepends each node's transform as a nested subtree. Takes up to three
    /// values `[<space>] [<rot-unit>] [<precision>]`: `space` is `local`
    /// (default) or `world`, `rot-unit` is `rad` (default) or `deg`, and
    /// `precision` is the decimal places (default `2`).
    #[arg(
        value_names = ["space", "rot-unit", "precision"],
        long = "show-transforms",
        num_args = 0..=3,
    )]
    show_transforms: Option<Vec<String>>,

    /// Appends each object's edit-grid origin, its build-volume min corner
    /// relative to the placing node. Takes up to two values
    /// `[<space>] [<precision>]`: `space` is `local` (default) or `world`, and
    /// `precision` is the decimal places (default `2`).
    #[arg(
        value_names = ["space", "precision"],
        long = "show-edit-origins",
        num_args = 0..=2,
    )]
    show_edit_origins: Option<Vec<String>>,

    /// Appends each object's edit-grid bounds as a `min`/`max` subtree, relative
    /// to the placing node. Takes an optional `[<precision>]`, the decimal
    /// places (default `2`).
    #[arg(value_names = ["precision"], long = "show-edit-bounds", num_args = 0..=1)]
    show_edit_bounds: Option<Vec<String>>,

    /// Appends each object's edit-grid extents (`max - min`). Takes an optional
    /// `[<precision>]`, the decimal places (default `2`).
    #[arg(value_names = ["precision"], long = "show-edit-extents", num_args = 0..=1)]
    show_edit_extents: Option<Vec<String>>,

    /// Appends each object's runtime-grid origin, its tight live-box min corner
    /// relative to the placing node. Takes up to two values
    /// `[<space>] [<precision>]`, read as `--show-edit-origins` reads them.
    #[arg(
        value_names = ["space", "precision"],
        long = "show-runtime-origins",
        num_args = 0..=2,
    )]
    show_runtime_origins: Option<Vec<String>>,

    /// Appends each object's runtime-grid bounds as a `min`/`max` subtree,
    /// relative to the placing node. Takes an optional `[<precision>]`, the
    /// decimal places (default `2`).
    #[arg(value_names = ["precision"], long = "show-runtime-bounds", num_args = 0..=1)]
    show_runtime_bounds: Option<Vec<String>>,

    /// Appends each object's runtime-grid extents (`max - min`). Takes an
    /// optional `[<precision>]`, the decimal places (default `2`).
    #[arg(value_names = ["precision"], long = "show-runtime-extents", num_args = 0..=1)]
    show_runtime_extents: Option<Vec<String>>,

    /// Appends each object's filled (live) voxel count as a `voxel-count` leaf.
    #[arg(value_name = "show-voxel-counts", long = "show-voxel-counts")]
    show_voxel_counts: bool,

    /// Appends each object's layers as a nested subtree, one child per layer
    /// showing its palette id and material count.
    #[arg(value_name = "show-layers", long = "show-layers")]
    show_layers: bool,
}

impl NodeList {
    /// Runs the command.
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let views = NodeListViews {
            transforms: parse(self.show_transforms.as_deref(), parse_transform_view)?,
            edit_origins: parse(self.show_edit_origins.as_deref(), parse_origin_view)?,
            edit_bounds: parse(self.show_edit_bounds.as_deref(), parse_precision_arg)?,
            edit_extents: parse(self.show_edit_extents.as_deref(), parse_precision_arg)?,
            runtime_origins: parse(self.show_runtime_origins.as_deref(), parse_origin_view)?,
            runtime_bounds: parse(self.show_runtime_bounds.as_deref(), parse_precision_arg)?,
            runtime_extents: parse(self.show_runtime_extents.as_deref(), parse_precision_arg)?,
            voxel_counts: self.show_voxel_counts,
            layers: self.show_layers,
        };

        // The collapse flags require a pattern, so clap guarantees they are false
        // when none is given; bundling them with the globs keeps that invariant.
        let pattern = (!self.patterns.is_empty()).then_some(PatternView {
            globs: self.patterns,
            collapse_ancestors: self.collapse_ancestors,
            collapse_descendants: self.collapse_descendants,
        });

        let from = self.input.resolve_format()?;

        let main: VoxMain = load_vox_main(&dependencies, from, &self.input.path)?;

        let options = NodeListOptions {
            pattern,
            layout: self.layout,
            collapse_instances: self.collapse_instances,
            views,
        };

        let output = node_list(&main, &options)?;

        Ok(dependencies.write_stdout(output.as_bytes())?)
    }
}

/// Runs `parse` over the flag's raw values when the flag was given, threading
/// the parse error out.
fn parse<T>(
    values: Option<&[String]>,
    parse: impl Fn(&[String]) -> Result<T>,
) -> Result<Option<T>> {
    values.map(parse).transpose()
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

/// Parses `[space] [precision]` into an [`OriginView`].
fn parse_origin_view(values: &[String]) -> Result<OriginView> {
    let world = parse_space(values.first())?;
    let precision = parse_precision(values.get(1))?;

    Ok(OriginView { world, precision })
}

/// Parses a lone `[precision]` value, defaulting to `2`.
fn parse_precision_arg(values: &[String]) -> Result<usize> {
    parse_precision(values.first())
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
        NodeList,
        node::node_list::node_list::{
            parse_origin_view, parse_precision_arg, parse_transform_view,
        },
    };
    use clap::Parser;
    use voxsmith::operations::node::NodeListLayout;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn a_collapse_flag_requires_a_pattern() {
        // Without a pattern, clap rejects the collapse flag rather than silently
        // ignoring it.
        assert!(NodeList::try_parse_from(["list", "in.voxj", "--collapse-ancestors"]).is_err());
        // With a pattern it parses.
        assert!(
            NodeList::try_parse_from(["list", "in.voxj", "door", "--collapse-ancestors"]).is_ok()
        );
    }

    #[test]
    fn layout_defaults_to_hierarchy_and_parses_the_json_values() {
        let list = NodeList::try_parse_from(["list", "in.voxj"]).unwrap();
        assert_eq!(list.layout, NodeListLayout::BoxHierarchy);

        let list =
            NodeList::try_parse_from(["list", "in.voxj", "--layout", "json-pretty"]).unwrap();
        assert_eq!(list.layout, NodeListLayout::JsonPretty);

        let list =
            NodeList::try_parse_from(["list", "in.voxj", "--layout", "json-compact"]).unwrap();
        assert_eq!(list.layout, NodeListLayout::JsonCompact);
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
    fn origin_view_parses_space_and_precision() {
        let view = parse_origin_view(&strings(&["world", "3"])).unwrap();
        assert!(view.world && view.precision == 3);
        let default = parse_origin_view(&[]).unwrap();
        assert!(!default.world && default.precision == 2);
    }

    #[test]
    fn precision_arg_defaults_to_two_and_parses_a_value() {
        assert!(parse_precision_arg(&[]).unwrap() == 2);
        assert!(parse_precision_arg(&strings(&["4"])).unwrap() == 4);
    }

    #[test]
    fn a_bad_space_or_unit_is_an_error() {
        assert!(parse_transform_view(&strings(&["sideways"])).is_err());
        assert!(parse_transform_view(&strings(&["local", "turns"])).is_err());
        assert!(parse_origin_view(&strings(&["local", "-1"])).is_err());
        assert!(parse_precision_arg(&strings(&["-1"])).is_err());
    }
}
