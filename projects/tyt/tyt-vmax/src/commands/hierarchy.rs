use crate::{Dependencies, Error, ResolvedNodeTransform, Result, VoxelMaxSceneNode};
use branded_id::U32Id;
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    f64::consts::PI,
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridHierarchyOptions, TreeGridLabel, TreeGridRenderHierarchy,
    TreeGridValue,
};

/// A node id in the [`TreeGrid`] being populated, distinct from the scene's
/// `usize` node indices.
type GridNodeId = U32Id<BTreeGridNode>;

/// The largest `--show-*` precision accepted, keeping float formatting below the
/// width the standard formatter can represent.
const MAX_PRECISION: usize = 255;

/// Prints the Voxel Max hierarchy as a tree, optionally filtered to selected
/// nodes and their subtrees.
#[derive(Clone, Debug, Parser)]
#[command(name = "hierarchy")]
pub struct Hierarchy {
    /// The input `.vmax` directory to inspect.
    #[arg(value_name = "input-vmax")]
    input_vmax: PathBuf,

    /// Optional gitignore-style patterns selecting hierarchy paths. When set,
    /// only matched nodes and their ancestors print, and a matched group brings
    /// in its whole subtree. A bare name matches at any depth, a slashed
    /// pattern anchors to a root, a trailing `/` matches groups only, and a
    /// leading `!` deselects. With none given the whole hierarchy prints.
    #[arg(value_name = "select")]
    select: Vec<String>,

    /// Hide the ancestor chain above each match behind an `ancestors` marker.
    /// Requires a pattern.
    #[arg(
        value_name = "collapse-ancestors",
        long = "collapse-ancestors",
        requires = "select"
    )]
    collapse_ancestors: bool,

    /// Hide the descendants of each match behind a `descendants` marker.
    /// Requires a pattern.
    #[arg(
        value_name = "collapse-descendants",
        long = "collapse-descendants",
        requires = "select"
    )]
    collapse_descendants: bool,

    /// Append each node's transform, with rotation as euler angles, as a
    /// nested subtree. An optional `=space,rot-unit,precision` packs the view,
    /// each part optional:
    /// 1. `space`: `local` (default) or `world`.
    /// 2. `rot-unit`: `rad` (default) or `deg`.
    /// 3. `precision`: decimal places, default 2.
    #[arg(value_name = "view", long = "show-transforms", num_args = 0..=1, require_equals = true)]
    show_transforms: Option<Vec<String>>,

    /// Append each node's authored voxel bounds as a `min`/`max` subtree, for
    /// nodes that have them. An optional `=precision` sets the decimal places
    /// (default 2).
    #[arg(value_name = "precision", long = "show-bounds", num_args = 0..=1, require_equals = true)]
    show_bounds: Option<Vec<String>>,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_vmax,
            select,
            collapse_ancestors,
            collapse_descendants,
            show_transforms,
            show_bounds,
        } = self;

        let show_transforms = parse_transform_view(show_transforms)?;
        let show_bounds = parse_precision(show_bounds)?;

        let bytes = dependencies.read_file(&input_vmax.join("scene.json"))?;
        let nodes = dependencies.scene_nodes(&bytes)?;

        // Child indices keyed by parent id (`None` at the root), sorted by name.
        let mut children: HashMap<Option<&str>, Vec<usize>> = HashMap::new();
        for (index, node) in nodes.iter().enumerate() {
            children
                .entry(node.parent_id.as_deref())
                .or_default()
                .push(index);
        }
        for kids in children.values_mut() {
            kids.sort_by(|&a, &b| nodes[a].name.cmp(&nodes[b].name));
        }

        // Each reachable node's path, parent index, and pre-order listing.
        let mut path_of = vec![String::new(); nodes.len()];
        let mut parent_of = vec![None; nodes.len()];
        let mut reachable = Vec::new();
        for &root in children.get(&None).unwrap_or(&Vec::new()) {
            assign_paths(
                &nodes,
                &children,
                root,
                None,
                "",
                &mut path_of,
                &mut parent_of,
                &mut reachable,
            );
        }

        let filtering = !select.is_empty();
        let (selected, visible, match_roots) = if filtering {
            select_nodes(
                &dependencies,
                &select,
                &nodes,
                &path_of,
                &parent_of,
                &reachable,
            )?
        } else {
            (HashSet::new(), HashSet::new(), HashSet::new())
        };

        // Resolve the local or world transform per node up front, while
        // `parent_of` is still in hand, so the renderer just formats them.
        let transforms = show_transforms
            .map(|view| dependencies.resolve_node_transforms(&nodes, &parent_of, view.world))
            .unwrap_or_default();

        let mut builder = Builder {
            nodes: &nodes,
            children,
            parent_of,
            selected,
            visible,
            match_roots,
            filtering,
            collapse_descendants,
            show_transforms,
            transforms,
            show_bounds,
            grid: TreeGrid::new(),
        };

        if collapse_ancestors && filtering {
            builder.build_collapsed_ancestors();
        } else {
            builder.build_tree();
        }

        let output = builder
            .grid
            .render_hierarchy(&TreeGridHierarchyOptions::default());
        dependencies.write_stdout(output.as_bytes())?;
        Ok(())
    }
}

/// Records `index`'s path (built from `prefix`) and parent, marks it reachable,
/// then recurses into its children in stored order.
#[allow(clippy::too_many_arguments)]
fn assign_paths(
    nodes: &[VoxelMaxSceneNode],
    children: &HashMap<Option<&str>, Vec<usize>>,
    index: usize,
    parent: Option<usize>,
    prefix: &str,
    path_of: &mut [String],
    parent_of: &mut [Option<usize>],
    reachable: &mut Vec<usize>,
) {
    let path = if prefix.is_empty() {
        nodes[index].name.clone()
    } else {
        format!("{prefix}/{}", nodes[index].name)
    };

    reachable.push(index);
    parent_of[index] = parent;

    if let Some(kids) = children.get(&Some(nodes[index].id.as_str())) {
        for &kid in kids {
            assign_paths(
                nodes,
                children,
                kid,
                Some(index),
                &path,
                path_of,
                parent_of,
                reachable,
            );
        }
    }

    path_of[index] = path;
}

/// Resolves the selection patterns into the `(selected, visible, match_roots)`
/// node-index sets. `selected` is every node the gitignore patterns reach,
/// `visible` adds their ancestors, and `match_roots` are the topmost selected
/// nodes. Keying on index, not path, keeps same-name siblings distinct.
fn select_nodes(
    dependencies: &impl Dependencies,
    select: &[String],
    nodes: &[VoxelMaxSceneNode],
    path_of: &[String],
    parent_of: &[Option<usize>],
    reachable: &[usize],
) -> Result<(HashSet<usize>, HashSet<usize>, HashSet<usize>)> {
    let candidates: Vec<(&str, bool)> = reachable
        .iter()
        .map(|&index| (path_of[index].as_str(), nodes[index].is_group))
        .collect();
    let patterns: Vec<&str> = select.iter().map(String::as_str).collect();
    let matched = dependencies.match_subtrees(&patterns, &candidates)?;

    let selected: HashSet<usize> = reachable
        .iter()
        .zip(matched.iter())
        .filter(|&(_, &m)| m)
        .map(|(&index, _)| index)
        .collect();

    if selected.is_empty() {
        return Err(Error::IO(IOError::new(
            ErrorKind::NotFound,
            format!("no node or object matched any of: {}", select.join(", ")),
        )));
    }

    let mut visible = selected.clone();
    for &index in &selected {
        let mut ancestor = parent_of[index];
        while let Some(node) = ancestor {
            visible.insert(node);
            ancestor = parent_of[node];
        }
    }

    let match_roots: HashSet<usize> = selected
        .iter()
        .copied()
        .filter(|&index| match parent_of[index] {
            Some(parent) => !selected.contains(&parent),
            None => true,
        })
        .collect();

    Ok((selected, visible, match_roots))
}

/// Resolved `--show-transforms` view: the space, rotation unit, and precision
/// to render each node's transform in.
#[derive(Clone, Copy, Debug)]
struct TransformView {
    /// Render in world space rather than local.
    world: bool,
    /// Render rotation in degrees rather than radians.
    degrees: bool,
    /// Decimal places for each component.
    precision: usize,
}

/// Parses `--show-transforms`: `None` when the flag is absent, else a
/// [`TransformView`]. `require_equals` delivers the arguments packed into one
/// `=`-attached value as `space,rot-unit,precision`, each part optional and
/// defaulting to `local`, `rad`, and `2`. Packing keeps the variadic `select`
/// patterns from being swallowed as flag values.
fn parse_transform_view(values: Option<Vec<String>>) -> Result<Option<TransformView>> {
    let Some(values) = values else {
        return Ok(None);
    };

    let packed = values.first().map(String::as_str).unwrap_or_default();
    let parts: Vec<&str> = packed.split(',').collect();

    let world = parse_space(parts.first().copied())?;
    let degrees = parse_rot_unit(parts.get(1).copied())?;
    let precision = parse_precision_value(parts.get(2).copied())?;

    Ok(Some(TransformView {
        world,
        degrees,
        precision,
    }))
}

/// Parses the packed `space`: `local` (also empty or absent) or `world`.
fn parse_space(value: Option<&str>) -> Result<bool> {
    match value {
        None | Some("" | "local") => Ok(false),
        Some("world") => Ok(true),
        Some(other) => Err(invalid_input(format!(
            "space must be 'local' or 'world', got '{other}'"
        ))),
    }
}

/// Parses the packed `rot-unit`: `rad` (also empty or absent) or `deg`.
fn parse_rot_unit(value: Option<&str>) -> Result<bool> {
    match value {
        None | Some("" | "rad") => Ok(false),
        Some("deg") => Ok(true),
        Some(other) => Err(invalid_input(format!(
            "rot-unit must be 'rad' or 'deg', got '{other}'"
        ))),
    }
}

/// Parses a `--show-bounds` flag's optional `=precision`: `None` when the flag
/// is absent, `Some(precision)` when present (default 2).
fn parse_precision(values: Option<Vec<String>>) -> Result<Option<usize>> {
    match values {
        None => Ok(None),
        Some(values) => Ok(Some(parse_precision_value(
            values.first().map(String::as_str),
        )?)),
    }
}

/// Parses a precision value, a non-negative integer defaulting to 2 when absent
/// or empty and capped at [`MAX_PRECISION`].
fn parse_precision_value(value: Option<&str>) -> Result<usize> {
    let precision = match value {
        None | Some("") => 2,
        Some(text) => text.parse::<usize>().map_err(|error| {
            invalid_input(format!("precision must be a non-negative integer: {error}"))
        })?,
    };

    if precision > MAX_PRECISION {
        return Err(invalid_input(format!(
            "precision must be at most {MAX_PRECISION}"
        )));
    }

    Ok(precision)
}

/// An invalid-input [`Error`] carrying `message`.
fn invalid_input(message: String) -> Error {
    Error::IO(IOError::new(ErrorKind::InvalidInput, message))
}

/// Populates the filtered hierarchy tree into a [`TreeGrid`]. The selection
/// sets hold node indices, so same-name siblings never conflate.
struct Builder<'a> {
    nodes: &'a [VoxelMaxSceneNode],
    children: HashMap<Option<&'a str>, Vec<usize>>,
    parent_of: Vec<Option<usize>>,
    selected: HashSet<usize>,
    visible: HashSet<usize>,
    match_roots: HashSet<usize>,
    filtering: bool,
    collapse_descendants: bool,
    show_transforms: Option<TransformView>,
    transforms: Vec<ResolvedNodeTransform>,
    show_bounds: Option<usize>,
    grid: TreeGrid,
}

impl Builder<'_> {
    /// Adds every visible root subtree in order.
    fn build_tree(&mut self) {
        let roots = self.children.get(&None).cloned().unwrap_or_default();
        for root in roots {
            if self.will_show(root) {
                self.build_node(root, None);
            }
        }
    }

    /// Adds each match root as a flat root list, behind an `ancestors` marker
    /// root when its ancestor chain is hidden.
    fn build_collapsed_ancestors(&mut self) {
        for node in self.collect_match_roots() {
            let parent = self.parent_of[node]
                .is_some()
                .then(|| self.grid.add_root(TreeGridLabel::bare("ancestors")));
            self.build_node(node, parent);
        }
    }

    /// The match-root node indices in pre-order.
    fn collect_match_roots(&self) -> Vec<usize> {
        let mut roots = Vec::new();
        let top = self.children.get(&None).cloned().unwrap_or_default();
        for node in top {
            self.collect_match_roots_from(node, &mut roots);
        }
        roots
    }

    fn collect_match_roots_from(&self, index: usize, roots: &mut Vec<usize>) {
        if self.match_roots.contains(&index) {
            roots.push(index);
        }
        if let Some(kids) = self.children.get(&Some(self.nodes[index].id.as_str())) {
            for &kid in kids {
                self.collect_match_roots_from(kid, roots);
            }
        }
    }

    /// Adds `index`'s node, then its transform and bounds subtrees, then the
    /// visible children or a `descendants` marker.
    fn build_node(&mut self, index: usize, parent: Option<GridNodeId>) {
        let (name, is_group) = {
            let node = &self.nodes[index];
            (node.name.clone(), node.is_group)
        };

        let grid_node = self.add_node(parent, TreeGridLabel::bare(name));
        let kind = if is_group { "(Group)" } else { "(Object)" };
        self.grid.node_mut(grid_node).annotation = Some(kind.to_owned());

        if self.show_transforms.is_some() {
            self.build_transform(index, grid_node);
        }
        if self.show_bounds.is_some() && self.nodes[index].bounds.is_some() {
            self.build_bounds(index, grid_node);
        }

        if is_group {
            let kids: Vec<usize> = self
                .children
                .get(&Some(self.nodes[index].id.as_str()))
                .map(|kids| {
                    kids.iter()
                        .copied()
                        .filter(|&kid| self.will_show(kid))
                        .collect()
                })
                .unwrap_or_default();

            let is_match_root = self.filtering && self.match_roots.contains(&index);
            if self.collapse_descendants && is_match_root && !kids.is_empty() {
                self.grid
                    .add_child(grid_node, TreeGridLabel::bare("descendants"));
            } else {
                for kid in kids {
                    self.build_node(kid, Some(grid_node));
                }
            }
        }
    }

    /// Adds a node under `parent`, or as a grid root when there is none.
    fn add_node(&mut self, parent: Option<GridNodeId>, label: TreeGridLabel) -> GridNodeId {
        match parent {
            Some(parent) => self.grid.add_child(parent, label),
            None => self.grid.add_root(label),
        }
    }

    /// Adds a leaf under `parent` carrying one pre-formatted value.
    fn add_value_leaf(&mut self, parent: GridNodeId, label: &str, text: String) {
        let leaf = self.grid.add_child(parent, TreeGridLabel::bare(label));
        self.grid.push_value(leaf, TreeGridValue::new(text));
    }

    /// Whether `index` renders: everything when not filtering, else a group when
    /// it leads to a selection and an object when it is itself selected. A
    /// matched group's descendants ride in because the subtree matcher put them
    /// in `selected`.
    fn will_show(&self, index: usize) -> bool {
        if !self.filtering {
            return true;
        }

        if self.nodes[index].is_group {
            self.visible.contains(&index)
        } else {
            self.selected.contains(&index)
        }
    }

    fn build_transform(&mut self, index: usize, parent: GridNodeId) {
        let Some(view) = self.show_transforms else {
            return;
        };
        let resolved = self.transforms[index];

        // The resolved rotation is euler radians; scale it to degrees
        // on request.
        let mut rotation = resolved.rotation;
        if view.degrees {
            let factor = 180.0 / PI;
            rotation = [
                rotation[0] * factor,
                rotation[1] * factor,
                rotation[2] * factor,
            ];
        }

        let precision = view.precision;
        let subtree = self
            .grid
            .add_child(parent, TreeGridLabel::bare("transform"));
        self.add_value_leaf(subtree, "position", fmt3(resolved.position, precision));
        self.add_value_leaf(subtree, "rotation", fmt3(rotation, precision));
        self.add_value_leaf(subtree, "scale", fmt3(resolved.scale, precision));
    }

    fn build_bounds(&mut self, index: usize, parent: GridNodeId) {
        let precision = self.show_bounds.unwrap_or(2);
        let Some((min, max)) = self.nodes[index].bounds else {
            return;
        };

        let subtree = self.grid.add_child(parent, TreeGridLabel::bare("bounds"));
        self.add_value_leaf(subtree, "min", fmt3(min, precision));
        self.add_value_leaf(subtree, "max", fmt3(max, precision));
    }
}

/// Formats a 3-vector as `[x, y, z]` with `precision` decimals.
fn fmt3(v: [f64; 3], precision: usize) -> String {
    format!(
        "[{:.prec$}, {:.prec$}, {:.prec$}]",
        v[0],
        v[1],
        v[2],
        prec = precision
    )
}

#[cfg(test)]
mod tests {
    use crate::commands::hierarchy::{
        parse_precision, parse_rot_unit, parse_space, parse_transform_view,
    };

    /// Wraps `values` as the flag's parsed `Some(..)` payload.
    fn packed(values: &[&str]) -> Option<Vec<String>> {
        Some(values.iter().map(|value| value.to_string()).collect())
    }

    #[test]
    fn transform_view_defaults_to_local_radians_precision_two() {
        // A bare flag parses to an empty value list.
        let view = parse_transform_view(Some(Vec::new())).unwrap().unwrap();
        assert!(!view.world && !view.degrees && view.precision == 2);
    }

    #[test]
    fn transform_view_parses_the_packed_parts() {
        let view = parse_transform_view(packed(&["world,deg,4"]))
            .unwrap()
            .unwrap();
        assert!(view.world && view.degrees && view.precision == 4);
    }

    #[test]
    fn transform_view_fills_defaults_for_omitted_parts() {
        // An empty middle part keeps the default rotation unit.
        let view = parse_transform_view(packed(&["world,,3"]))
            .unwrap()
            .unwrap();
        assert!(view.world && !view.degrees && view.precision == 3);
    }

    #[test]
    fn transform_view_is_none_when_the_flag_is_absent() {
        assert!(parse_transform_view(None).unwrap().is_none());
    }

    #[test]
    fn a_bad_space_unit_or_precision_is_an_error() {
        assert!(parse_transform_view(packed(&["sideways"])).is_err());
        assert!(parse_transform_view(packed(&["local,turns"])).is_err());
        assert!(parse_transform_view(packed(&["local,rad,-1"])).is_err());
    }

    #[test]
    fn space_and_rot_unit_parse_their_tokens() {
        assert!(!parse_space(None).unwrap() && !parse_space(Some("local")).unwrap());
        assert!(parse_space(Some("world")).unwrap());
        assert!(!parse_rot_unit(Some("rad")).unwrap() && parse_rot_unit(Some("deg")).unwrap());
    }

    #[test]
    fn precision_flag_defaults_and_parses() {
        assert!(parse_precision(None).unwrap().is_none());
        assert!(parse_precision(Some(Vec::new())).unwrap() == Some(2));
        assert!(parse_precision(packed(&["4"])).unwrap() == Some(4));
    }
}
