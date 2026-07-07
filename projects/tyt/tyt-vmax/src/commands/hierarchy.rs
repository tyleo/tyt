use crate::{Dependencies, Error, Result, VoxelMaxSceneNode};
use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};

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

    /// Append each node's local transform as a nested subtree. An optional
    /// `=precision` sets the decimal places (default 2). Rotation is axis-angle.
    #[arg(value_name = "precision", long = "show-transforms", num_args = 0..=1, require_equals = true)]
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

        let show_transforms = parse_precision(show_transforms)?;
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

        let mut renderer = Renderer {
            nodes: &nodes,
            children,
            parent_of,
            selected,
            visible,
            match_roots,
            filtering,
            collapse_descendants,
            show_transforms,
            show_bounds,
            output: String::new(),
        };

        if collapse_ancestors && filtering {
            renderer.render_collapsed_ancestors();
        } else {
            renderer.render_tree();
        }

        dependencies.write_stdout(renderer.output.as_bytes())?;
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

/// Parses a `--show-*` flag's optional `=precision`: `None` when the flag is
/// absent, `Some(precision)` when present (default 2).
fn parse_precision(values: Option<Vec<String>>) -> Result<Option<usize>> {
    let Some(values) = values else {
        return Ok(None);
    };

    let precision = match values.first() {
        Some(value) => value.parse::<usize>().map_err(|error| {
            Error::IO(IOError::new(
                ErrorKind::InvalidInput,
                format!("precision must be a non-negative integer: {error}"),
            ))
        })?,
        None => 2,
    };

    if precision > MAX_PRECISION {
        return Err(Error::IO(IOError::new(
            ErrorKind::InvalidInput,
            format!("precision must be at most {MAX_PRECISION}"),
        )));
    }

    Ok(Some(precision))
}

/// A row appended under a node, in render order.
enum Item {
    /// The `transform` subtree.
    Transform,
    /// The `bounds` subtree.
    Bounds,
    /// The `descendants` collapse marker.
    Descendants,
    /// A real child node by index.
    Child(usize),
}

/// Renders the filtered hierarchy tree into an output string. The selection sets
/// hold node indices, so same-name siblings never conflate.
struct Renderer<'a> {
    nodes: &'a [VoxelMaxSceneNode],
    children: HashMap<Option<&'a str>, Vec<usize>>,
    parent_of: Vec<Option<usize>>,
    selected: HashSet<usize>,
    visible: HashSet<usize>,
    match_roots: HashSet<usize>,
    filtering: bool,
    collapse_descendants: bool,
    show_transforms: Option<usize>,
    show_bounds: Option<usize>,
    output: String,
}

impl Renderer<'_> {
    /// Renders every visible root subtree in order.
    fn render_tree(&mut self) {
        let roots = self.children.get(&None).cloned().unwrap_or_default();
        let shown: Vec<usize> = roots
            .into_iter()
            .filter(|&root| self.will_show(root))
            .collect();

        let count = shown.len();
        for (index, root) in shown.into_iter().enumerate() {
            self.render_node(root, "", index + 1 == count);
        }
    }

    /// Renders each match root as a flat list, prefixed with an `ancestors`
    /// marker when its ancestor chain is hidden.
    fn render_collapsed_ancestors(&mut self) {
        let roots = self.collect_match_roots();
        let count = roots.len();

        for (index, node) in roots.into_iter().enumerate() {
            let is_last = index + 1 == count;
            if self.parent_of[node].is_some() {
                let connector = if is_last { '└' } else { '├' };
                let extension = if is_last { "  " } else { "│ " };
                self.output.push_str(&format!("{connector} ancestors\n"));
                self.render_node(node, extension, true);
            } else {
                self.render_node(node, "", is_last);
            }
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

    /// Prints `index`'s row, then its transform, bounds, and child rows.
    fn render_node(&mut self, index: usize, prefix: &str, is_last: bool) {
        let (name, is_group) = {
            let node = &self.nodes[index];
            (node.name.clone(), node.is_group)
        };

        let connector = if is_last { '└' } else { '├' };
        let kind = if is_group { "Group" } else { "Object" };
        self.output
            .push_str(&format!("{prefix}{connector} {name} ({kind})\n"));

        let child_prefix = format!("{prefix}{}", if is_last { "  " } else { "│ " });
        let is_match_root = self.filtering && self.match_roots.contains(&index);

        let items = self.child_items(index, is_group, is_match_root);
        let count = items.len();
        for (position, item) in items.into_iter().enumerate() {
            let item_last = position + 1 == count;
            match item {
                Item::Transform => self.render_transform(index, &child_prefix, item_last),
                Item::Bounds => self.render_bounds(index, &child_prefix, item_last),
                Item::Descendants => {
                    let connector = if item_last { '└' } else { '├' };
                    self.output
                        .push_str(&format!("{child_prefix}{connector} descendants\n"));
                }
                Item::Child(child) => self.render_node(child, &child_prefix, item_last),
            }
        }
    }

    /// The rows to render under `index`: an optional transform and bounds
    /// subtree, then the visible children or a `descendants` marker.
    fn child_items(&self, index: usize, is_group: bool, is_match_root: bool) -> Vec<Item> {
        let mut items = Vec::new();

        if self.show_transforms.is_some() {
            items.push(Item::Transform);
        }
        if self.show_bounds.is_some() && self.nodes[index].bounds.is_some() {
            items.push(Item::Bounds);
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

            if self.collapse_descendants && is_match_root && !kids.is_empty() {
                items.push(Item::Descendants);
            } else {
                items.extend(kids.into_iter().map(Item::Child));
            }
        }

        items
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

    fn render_transform(&mut self, index: usize, prefix: &str, is_last: bool) {
        let precision = self.show_transforms.unwrap_or(2);
        let (position, rotation, scale) = {
            let node = &self.nodes[index];
            (node.position, node.rotation, node.scale)
        };

        let connector = if is_last { '└' } else { '├' };
        let inner = format!("{prefix}{}", if is_last { "  " } else { "│ " });
        self.output
            .push_str(&format!("{prefix}{connector} transform\n"));
        self.output.push_str(&format!(
            "{inner}├ position: {}\n",
            fmt3(position, precision)
        ));
        self.output.push_str(&format!(
            "{inner}├ rotation: {}\n",
            fmt4(rotation, precision)
        ));
        self.output
            .push_str(&format!("{inner}└ scale: {}\n", fmt3(scale, precision)));
    }

    fn render_bounds(&mut self, index: usize, prefix: &str, is_last: bool) {
        let precision = self.show_bounds.unwrap_or(2);
        let Some((min, max)) = self.nodes[index].bounds else {
            return;
        };

        let connector = if is_last { '└' } else { '├' };
        let inner = format!("{prefix}{}", if is_last { "  " } else { "│ " });
        self.output
            .push_str(&format!("{prefix}{connector} bounds\n"));
        self.output
            .push_str(&format!("{inner}├ min: {}\n", fmt3(min, precision)));
        self.output
            .push_str(&format!("{inner}└ max: {}\n", fmt3(max, precision)));
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

/// Formats a 4-vector as `[x, y, z, w]` with `precision` decimals.
fn fmt4(v: [f64; 4], precision: usize) -> String {
    format!(
        "[{:.prec$}, {:.prec$}, {:.prec$}, {:.prec$}]",
        v[0],
        v[1],
        v[2],
        v[3],
        prec = precision
    )
}
