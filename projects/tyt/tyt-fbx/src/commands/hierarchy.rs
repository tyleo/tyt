use crate::{
    Dependencies, Error, HierarchyBounds, HierarchyEntry, HierarchyTransform, Result, utilities,
};
use branded_id::U32Id;
use clap::Parser;
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    io::{Error as IOError, ErrorKind},
    path::PathBuf,
};
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridHierarchyOptions, TreeGridLabel, TreeGridRenderHierarchy,
};
use treeselect::TreeSelection;

/// A node id in the [`TreeGrid`] being populated, distinct from the
/// entries' `usize` indices.
type GridNodeId = U32Id<BTreeGridNode>;

/// Prints the FBX object hierarchy as a tree with box-drawing glyphs,
/// showing each object's name and type.
#[derive(Clone, Debug, Parser)]
pub struct Hierarchy {
    /// The input FBX file to inspect.
    #[arg(value_name = "input-fbx")]
    input_fbx: PathBuf,

    /// Optional gitignore-style patterns selecting object hierarchy paths. When
    /// set, only matched objects and their ancestors are printed, or only
    /// matched objects when `--collapse-ancestors` is used. A bare name matches
    /// at any depth, a slashed pattern anchors to a scene root; `**/name/**`
    /// selects a whole subtree. With none given the whole hierarchy prints.
    #[arg(value_name = "select")]
    select: Vec<String>,

    /// If set, prepend each object's transform (position, rotation, scale)
    /// as a nested subtree. Accepts up to three positional values:
    /// `[<space>] [<rot-unit>] [<precision>]`. `space` is `local` (default)
    /// or `world`. `rot-unit` is `rad` (default) or `deg`.
    /// `precision` is the decimal precision used to align vector components
    /// (default 2).
    #[arg(
        long = "show-transforms",
        value_names = ["space", "rot-unit", "precision"],
        num_args = 0..=3,
    )]
    show_transforms: Option<Vec<String>>,

    /// If set, append an axis-aligned bounding-box subtree for each object,
    /// aggregating the object's own mesh with all descendant meshes.
    /// Accepts up to three positional values:
    /// `[<space>] [<precision>] [<scale>]`. `space` is `local` (default) or
    /// `world`. `precision` is the decimal precision (default 2). `scale` is
    /// `no-scale` (default) or `scale`; when `scale`, the object's local
    /// scale is baked into local-space output (no effect in world space).
    #[arg(
        long = "show-bounds",
        value_names = ["space", "precision", "scale"],
        num_args = 0..=3,
    )]
    show_bounds: Option<Vec<String>>,

    /// If set, append an AABB-extents subtree (`max - min`) for each object,
    /// aggregating the object's own mesh with all descendant meshes.
    /// Accepts up to three positional values:
    /// `[<space>] [<precision>] [<scale>]`. Same semantics as `--show-bounds`.
    #[arg(
        long = "show-extents",
        value_names = ["space", "precision", "scale"],
        num_args = 0..=3,
    )]
    show_extents: Option<Vec<String>>,

    /// When set with `--select`, the ancestor chain above each matched
    /// object is hidden and replaced with an `(ANCESTORS)` marker printed
    /// directly above the matched object, omitted when the matched object
    /// is a scene root. Has no effect when `--select` is omitted.
    #[arg(value_name = "collapse-ancestors", long = "collapse-ancestors")]
    collapse_ancestors: bool,

    /// When set with `--select`, the descendants of each matched object
    /// are hidden and replaced with a `(DESCENDANTS)` marker printed as a
    /// child subtree of the matched object, omitted when the matched
    /// object has no descendants. Has no effect when `--select` is omitted.
    #[arg(value_name = "collapse-descendants", long = "collapse-descendants")]
    collapse_descendants: bool,
}

impl Hierarchy {
    pub fn execute(self, dependencies: impl Dependencies) -> Result<()> {
        let Hierarchy {
            input_fbx,
            select,
            show_transforms,
            show_bounds,
            show_extents,
            collapse_ancestors,
            collapse_descendants,
        } = self;

        let (show_xfm, xfm_prec, xfm_world, xfm_deg) = pack_transforms(show_transforms)?;
        let (show_bnd, bnd_prec, bnd_world, bnd_scale) = pack_bounds(show_bounds)?;
        let (show_ext, ext_prec, ext_world, ext_scale) = pack_bounds(show_extents)?;

        let args: [&OsStr; 13] = [
            input_fbx.as_ref(),
            show_xfm,
            xfm_prec.as_ref(),
            xfm_world,
            xfm_deg,
            show_bnd,
            bnd_prec.as_ref(),
            bnd_world,
            bnd_scale,
            show_ext,
            ext_prec.as_ref(),
            ext_world,
            ext_scale,
        ];
        let stdout =
            dependencies.exec_temp_blender_script(&utilities::FBX_HIERARCHY_JSON_PY, args)?;
        let json = utilities::extract_json(&stdout, b'[', b']')?;
        let entries = dependencies.parse_hierarchy_payloads_json(json)?;

        let output = render_tree(
            &dependencies,
            &entries,
            &select,
            collapse_ancestors,
            collapse_descendants,
        )?;
        dependencies.write_stdout(output.as_bytes())?;

        Ok(())
    }
}

/// Resolves `select` over the entries and renders the hierarchy tree.
fn render_tree(
    dependencies: &impl Dependencies,
    entries: &[HierarchyEntry],
    select: &[String],
    collapse_ancestors: bool,
    collapse_descendants: bool,
) -> Result<String> {
    let parents = parent_indices(entries);
    let (roots, children) = child_indices(&parents);

    let selection = if select.is_empty() {
        None
    } else {
        Some(resolve_selection(dependencies, entries, select, &parents)?)
    };

    let mut builder = Builder {
        entries,
        children: &children,
        selection: selection.as_ref(),
        collapse_descendants,
        grid: TreeGrid::new(),
    };

    if collapse_ancestors && selection.is_some() {
        builder.build_collapsed_ancestors(&parents);
    } else {
        builder.build_tree(&roots);
    }

    Ok(builder
        .grid
        .render_hierarchy(&TreeGridHierarchyOptions::default()))
}

/// Matches `select` over the entry paths; errors when nothing matches.
fn resolve_selection(
    dependencies: &impl Dependencies,
    entries: &[HierarchyEntry],
    select: &[String],
    parents: &[Option<usize>],
) -> Result<TreeSelection> {
    let candidate_paths: Vec<&str> = entries.iter().map(|entry| entry.path.as_str()).collect();
    let matched = utilities::match_hierarchy_paths(dependencies, select, &candidate_paths)?;

    if !matched.contains(&true) {
        return Err(Error::IO(IOError::new(
            ErrorKind::NotFound,
            format!("no object matched any of: {}", select.join(", ")),
        )));
    }

    Ok(TreeSelection::from_matches(matched, parents))
}

/// Per-entry parent indices, resolved from the `/`-joined paths. Entries
/// arrive in pre-order, so a parent always precedes its children.
fn parent_indices(entries: &[HierarchyEntry]) -> Vec<Option<usize>> {
    let mut index_of: HashMap<&str, usize> = HashMap::new();
    let mut parents = Vec::with_capacity(entries.len());

    for (index, entry) in entries.iter().enumerate() {
        let parent = entry
            .path
            .rsplit_once('/')
            .and_then(|(parent_path, _)| index_of.get(parent_path).copied());
        parents.push(parent);
        index_of.insert(entry.path.as_str(), index);
    }

    parents
}

/// Root indices and per-entry child indices, both in entry order.
fn child_indices(parents: &[Option<usize>]) -> (Vec<usize>, Vec<Vec<usize>>) {
    let mut roots = Vec::new();
    let mut children = vec![Vec::new(); parents.len()];

    for (index, parent) in parents.iter().enumerate() {
        match parent {
            Some(parent) => children[*parent].push(index),
            None => roots.push(index),
        }
    }

    (roots, children)
}

/// Populates the filtered hierarchy tree into a [`TreeGrid`]. A matched
/// object shows its whole subtree, so the walk threads an in-match flag
/// down and consults visibility only outside match subtrees.
struct Builder<'a> {
    entries: &'a [HierarchyEntry],
    children: &'a [Vec<usize>],
    selection: Option<&'a TreeSelection>,
    collapse_descendants: bool,
    grid: TreeGrid,
}

impl Builder<'_> {
    /// Adds every visible root subtree in order.
    fn build_tree(&mut self, roots: &[usize]) {
        for &root in roots {
            let visible = match self.selection {
                Some(selection) => selection.visible()[root],
                None => true,
            };
            if visible {
                self.build_node(root, None, false);
            }
        }
    }

    /// Adds each match as its own root listing, in entry order, behind an
    /// `(ANCESTORS)` marker root when its ancestor chain is hidden.
    fn build_collapsed_ancestors(&mut self, parents: &[Option<usize>]) {
        let Some(selection) = self.selection else {
            return;
        };

        for (index, parent) in parents.iter().enumerate() {
            if !selection.selected()[index] {
                continue;
            }

            let marker = parent
                .is_some()
                .then(|| self.grid.add_root(TreeGridLabel::bare("(ANCESTORS)")));
            self.build_node(index, marker, true);
        }
    }

    /// Adds `index`'s object node, its payload subtrees, then its visible
    /// children or a `(DESCENDANTS)` marker.
    fn build_node(&mut self, index: usize, parent: Option<GridNodeId>, in_match: bool) {
        let entry = &self.entries[index];
        let node = match parent {
            Some(parent) => self
                .grid
                .add_child(parent, TreeGridLabel::bare(entry.name.as_str())),
            None => self.grid.add_root(TreeGridLabel::bare(entry.name.as_str())),
        };
        self.grid.node_mut(node).annotation = Some(format!("({})", entry.object_type));

        let is_matched = self
            .selection
            .is_some_and(|selection| selection.selected()[index]);
        let in_match = in_match || is_matched;

        if let Some(transform) = &entry.transform {
            self.build_transform(transform, node);
        }
        if let Some(extents) = &entry.extents {
            self.add_vector(node, extents, "(EXTENTS)");
        }
        if let Some(bounds) = &entry.bounds {
            self.build_bounds(bounds, node);
        }

        let kids = &self.children[index];
        if in_match && self.collapse_descendants && !kids.is_empty() {
            self.grid
                .add_child(node, TreeGridLabel::bare("(DESCENDANTS)"));
            return;
        }

        for &kid in kids {
            let show = match self.selection {
                Some(selection) => in_match || selection.visible()[kid],
                None => true,
            };
            if show {
                self.build_node(kid, Some(node), in_match);
            }
        }
    }

    /// Adds the `(TRANSFORM)` subtree with its tagged vector lines.
    fn build_transform(&mut self, transform: &HierarchyTransform, parent: GridNodeId) {
        let subtree = self
            .grid
            .add_child(parent, TreeGridLabel::bare("(TRANSFORM)"));
        self.add_vector(subtree, &transform.position, "(POSITION)");
        self.add_vector(subtree, &transform.rotation, "(ROTATION)");
        self.add_vector(subtree, &transform.scale, "(SCALE)");
    }

    /// Adds the `(BOUNDS)` subtree with one min/max line per axis.
    fn build_bounds(&mut self, bounds: &HierarchyBounds, parent: GridNodeId) {
        let subtree = self.grid.add_child(parent, TreeGridLabel::bare("(BOUNDS)"));
        let axes = ["X", "Y", "Z"]
            .into_iter()
            .zip(&bounds.min)
            .zip(&bounds.max);
        for ((axis, min), max) in axes {
            let line = self.grid.add_child(
                subtree,
                TreeGridLabel::bare(format!("{{ \"Min{axis}\": {min}, \"Max{axis}\": {max} }}")),
            );
            self.grid.node_mut(line).annotation = Some(format!("({axis}-BOUNDS)"));
        }
    }

    /// Adds a vector line under `parent`: the component text as the
    /// label, `tag` as the annotation.
    fn add_vector(&mut self, parent: GridNodeId, components: &[String; 3], tag: &str) {
        let [x, y, z] = components;
        let line = self.grid.add_child(
            parent,
            TreeGridLabel::bare(format!("{{ \"X\": {x}, \"Y\": {y}, \"Z\": {z} }}")),
        );
        self.grid.node_mut(line).annotation = Some(tag.to_owned());
    }
}

fn pack_transforms(
    values: Option<Vec<String>>,
) -> Result<(&'static OsStr, OsString, &'static OsStr, &'static OsStr)> {
    match values {
        Some(values) => {
            let (p, w, d) = parse_transform_args(values)?;
            Ok((OsStr::new("true"), p, w, d))
        }
        None => Ok((
            OsStr::new("false"),
            OsString::from("2"),
            OsStr::new("false"),
            OsStr::new("false"),
        )),
    }
}

fn pack_bounds(
    values: Option<Vec<String>>,
) -> Result<(&'static OsStr, OsString, &'static OsStr, &'static OsStr)> {
    match values {
        Some(values) => {
            let (p, w, s) = parse_bounds_args(values)?;
            Ok((OsStr::new("true"), p, w, s))
        }
        None => Ok((
            OsStr::new("false"),
            OsString::from("2"),
            OsStr::new("false"),
            OsStr::new("false"),
        )),
    }
}

fn parse_transform_args(values: Vec<String>) -> Result<(OsString, &'static OsStr, &'static OsStr)> {
    let space = values.first().map(String::as_str).unwrap_or("local");
    let is_world = parse_space(space)?;

    let rot_unit = values.get(1).map(String::as_str).unwrap_or("rad");
    let is_degrees: &'static OsStr = match rot_unit {
        "rad" => OsStr::new("false"),
        "deg" => OsStr::new("true"),
        other => {
            return Err(
                IOError::other(format!("rot-unit must be 'rad' or 'deg', got '{other}'")).into(),
            );
        }
    };

    let precision_str = values.get(2).map(String::as_str).unwrap_or("2");
    parse_precision(precision_str)?;

    Ok((OsString::from(precision_str), is_world, is_degrees))
}

fn parse_bounds_args(values: Vec<String>) -> Result<(OsString, &'static OsStr, &'static OsStr)> {
    let space = values.first().map(String::as_str).unwrap_or("local");
    let is_world = parse_space(space)?;

    let precision_str = values.get(1).map(String::as_str).unwrap_or("2");
    parse_precision(precision_str)?;

    let scale = values.get(2).map(String::as_str).unwrap_or("no-scale");
    let apply_scale: &'static OsStr = match scale {
        "no-scale" => OsStr::new("false"),
        "scale" => OsStr::new("true"),
        other => {
            return Err(IOError::other(format!(
                "scale must be 'no-scale' or 'scale', got '{other}'"
            ))
            .into());
        }
    };

    Ok((OsString::from(precision_str), is_world, apply_scale))
}

fn parse_space(space: &str) -> Result<&'static OsStr> {
    match space {
        "local" => Ok(OsStr::new("false")),
        "world" => Ok(OsStr::new("true")),
        other => {
            Err(IOError::other(format!("space must be 'local' or 'world', got '{other}'")).into())
        }
    }
}

fn parse_precision(s: &str) -> Result<()> {
    s.parse::<u32>()
        .map_err(|e| IOError::other(format!("precision must be a non-negative integer: {e}")))?;
    Ok(())
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        Dependencies, DependenciesImpl, HierarchyEntry, Result, commands::hierarchy::render_tree,
    };

    /// A structure-only payload: a rig with a nested arm beside a second
    /// root.
    const STRUCTURE_JSON: &str = r#"[
        {"name": "Rig", "path": "Rig", "type": "EMPTY"},
        {"name": "Arm", "path": "Rig/Arm", "type": "EMPTY"},
        {"name": "Hand", "path": "Rig/Arm/Hand", "type": "MESH"},
        {"name": "Body", "path": "Rig/Body", "type": "MESH"},
        {"name": "Stage", "path": "Stage", "type": "MESH"}
    ]"#;

    /// A payload form: transform, bounds, and extents on a root and its
    /// mesh child, and a geometry-less sibling with only a transform.
    const PAYLOAD_JSON: &str = r#"[
        {
            "name": "Rig", "path": "Rig", "type": "EMPTY",
            "transform": {
                "position": ["0.00", "1.00", "0.00"],
                "rotation": ["0.00", "0.00", "0.00"],
                "scale": ["1.00", "1.00", "1.00"]
            },
            "bounds": {
                "min": ["-1.00", "0.00", "-1.00"],
                "max": ["1.00", "2.00", "1.00"]
            },
            "extents": ["2.00", "2.00", "2.00"]
        },
        {
            "name": "Hand", "path": "Rig/Hand", "type": "MESH",
            "transform": {
                "position": ["0.50", "0.00", "0.00"],
                "rotation": ["0.00", "0.00", "1.57"],
                "scale": ["1.00", "1.00", "1.00"]
            },
            "bounds": {
                "min": ["-1.00", "0.00", "-1.00"],
                "max": ["1.00", "2.00", "1.00"]
            },
            "extents": ["2.00", "2.00", "2.00"]
        },
        {
            "name": "Widget", "path": "Widget", "type": "EMPTY",
            "transform": {
                "position": ["3.00", "0.00", "0.00"],
                "rotation": ["0.00", "0.00", "0.00"],
                "scale": ["2.00", "2.00", "2.00"]
            }
        }
    ]"#;

    fn parse(json: &str) -> Vec<HierarchyEntry> {
        DependenciesImpl
            .parse_hierarchy_payloads_json(json.as_bytes())
            .unwrap()
    }

    fn render(
        entries: &[HierarchyEntry],
        select: &[&str],
        collapse_ancestors: bool,
        collapse_descendants: bool,
    ) -> Result<String> {
        let select: Vec<String> = select.iter().map(|pattern| pattern.to_string()).collect();
        render_tree(
            &DependenciesImpl,
            entries,
            &select,
            collapse_ancestors,
            collapse_descendants,
        )
    }

    fn expected(lines: &[&str]) -> String {
        let mut expected = lines.join("\n");
        expected.push('\n');
        expected
    }

    #[test]
    fn the_full_tree_renders_every_object_with_its_type() {
        let entries = parse(STRUCTURE_JSON);
        let output = render(&entries, &[], false, false).unwrap();
        assert_eq!(
            output,
            expected(&[
                "├ Rig (EMPTY)",
                "│ ├ Arm (EMPTY)",
                "│ │ └ Hand (MESH)",
                "│ └ Body (MESH)",
                "└ Stage (MESH)",
            ])
        );
    }

    #[test]
    fn payload_subtrees_render_in_transform_extents_bounds_order() {
        let entries = parse(PAYLOAD_JSON);
        let output = render(&entries, &[], false, false).unwrap();
        assert_eq!(
            output,
            expected(&[
                "├ Rig (EMPTY)",
                "│ ├ (TRANSFORM)",
                "│ │ ├ { \"X\": 0.00, \"Y\": 1.00, \"Z\": 0.00 } (POSITION)",
                "│ │ ├ { \"X\": 0.00, \"Y\": 0.00, \"Z\": 0.00 } (ROTATION)",
                "│ │ └ { \"X\": 1.00, \"Y\": 1.00, \"Z\": 1.00 } (SCALE)",
                "│ ├ { \"X\": 2.00, \"Y\": 2.00, \"Z\": 2.00 } (EXTENTS)",
                "│ ├ (BOUNDS)",
                "│ │ ├ { \"MinX\": -1.00, \"MaxX\": 1.00 } (X-BOUNDS)",
                "│ │ ├ { \"MinY\": 0.00, \"MaxY\": 2.00 } (Y-BOUNDS)",
                "│ │ └ { \"MinZ\": -1.00, \"MaxZ\": 1.00 } (Z-BOUNDS)",
                "│ └ Hand (MESH)",
                "│   ├ (TRANSFORM)",
                "│   │ ├ { \"X\": 0.50, \"Y\": 0.00, \"Z\": 0.00 } (POSITION)",
                "│   │ ├ { \"X\": 0.00, \"Y\": 0.00, \"Z\": 1.57 } (ROTATION)",
                "│   │ └ { \"X\": 1.00, \"Y\": 1.00, \"Z\": 1.00 } (SCALE)",
                "│   ├ { \"X\": 2.00, \"Y\": 2.00, \"Z\": 2.00 } (EXTENTS)",
                "│   └ (BOUNDS)",
                "│     ├ { \"MinX\": -1.00, \"MaxX\": 1.00 } (X-BOUNDS)",
                "│     ├ { \"MinY\": 0.00, \"MaxY\": 2.00 } (Y-BOUNDS)",
                "│     └ { \"MinZ\": -1.00, \"MaxZ\": 1.00 } (Z-BOUNDS)",
                "└ Widget (EMPTY)",
                "  └ (TRANSFORM)",
                "    ├ { \"X\": 3.00, \"Y\": 0.00, \"Z\": 0.00 } (POSITION)",
                "    ├ { \"X\": 0.00, \"Y\": 0.00, \"Z\": 0.00 } (ROTATION)",
                "    └ { \"X\": 2.00, \"Y\": 2.00, \"Z\": 2.00 } (SCALE)",
            ])
        );
    }

    #[test]
    fn a_selection_prunes_to_matches_their_ancestors_and_subtrees() {
        let entries = parse(STRUCTURE_JSON);
        let output = render(&entries, &["Arm"], false, false).unwrap();
        assert_eq!(
            output,
            expected(&["└ Rig (EMPTY)", "  └ Arm (EMPTY)", "    └ Hand (MESH)",])
        );
    }

    #[test]
    fn collapse_descendants_replaces_a_matched_subtree_with_a_marker() {
        let entries = parse(STRUCTURE_JSON);
        let output = render(&entries, &["Arm"], false, true).unwrap();
        assert_eq!(
            output,
            expected(&["└ Rig (EMPTY)", "  └ Arm (EMPTY)", "    └ (DESCENDANTS)",])
        );
    }

    #[test]
    fn collapse_ancestors_lists_each_match_behind_a_marker_root() {
        let entries = parse(STRUCTURE_JSON);
        let output = render(&entries, &["Arm", "Stage"], true, false).unwrap();
        assert_eq!(
            output,
            expected(&[
                "├ (ANCESTORS)",
                "│ └ Arm (EMPTY)",
                "│   └ Hand (MESH)",
                "└ Stage (MESH)",
            ])
        );
    }

    #[test]
    fn every_match_gets_its_own_collapsed_listing() {
        let entries = parse(STRUCTURE_JSON);
        let output = render(&entries, &["Arm", "Hand"], true, false).unwrap();
        assert_eq!(
            output,
            expected(&[
                "├ (ANCESTORS)",
                "│ └ Arm (EMPTY)",
                "│   └ Hand (MESH)",
                "└ (ANCESTORS)",
                "  └ Hand (MESH)",
            ])
        );
    }

    #[test]
    fn collapse_flags_without_a_selection_render_the_full_tree() {
        let entries = parse(STRUCTURE_JSON);
        let output = render(&entries, &[], true, true).unwrap();
        assert_eq!(
            output,
            expected(&[
                "├ Rig (EMPTY)",
                "│ ├ Arm (EMPTY)",
                "│ │ └ Hand (MESH)",
                "│ └ Body (MESH)",
                "└ Stage (MESH)",
            ])
        );
    }

    #[test]
    fn an_unmatched_selection_is_an_error() {
        let entries = parse(STRUCTURE_JSON);
        let error = render(&entries, &["Nope"], false, false).unwrap_err();
        assert!(error.to_string().contains("no object matched any of: Nope"));
    }
}
