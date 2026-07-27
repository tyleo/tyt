use crate::{
    Format, Result,
    commands::{HierarchyShowLayout, HierarchyViews, PatternView},
    implementation,
};
use branded_id::{IdVec, U32Id};
use pathspec::GitIgnoreRegex;
use std::{
    collections::HashSet,
    f64::consts::PI,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use treegrid::{
    BTreeGridNode, TreeGrid, TreeGridHierarchyOptions, TreeGridLabel, TreeGridRenderHierarchy,
    TreeGridRenderJson, TreeGridValue,
};
use treeselect::TreeSelection;
use ty_math::{TyQuaternionExt, TyTransformF64, TyVector3F64};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxMain, VoxObject};

/// A hierarchy-node id in the loaded [`VoxMain`], aliased so signatures stay
/// short and a node id never mixes with an object id.
type NodeId = U32Id<BVoxHierarchyNode>;

/// An object id in the loaded [`VoxMain`], aliased alongside [`NodeId`] to keep
/// the node-versus-object distinction in the type system.
type ObjectId = U32Id<BVoxObject>;

/// A node id in the [`TreeGrid`] being populated, aliased beside [`NodeId`] and
/// [`ObjectId`] so the three id spaces stay distinct in signatures.
type GridNodeId = U32Id<BTreeGridNode>;

/// Loads the voxel file at `input` and prints its scene graph under `layout`.
pub fn hierarchy_show(
    input: &Path,
    from: Option<Format>,
    pattern: Option<PatternView>,
    layout: HierarchyShowLayout,
    collapse_instances: bool,
    views: HierarchyViews,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let options = RenderOptions {
        pattern,
        layout,
        collapse_instances,
        views,
    };

    let output = render(&state, &options)?;

    implementation::write_stdout(output.as_bytes())
}

/// The knobs `render` reads: an optional node-path glob with its collapse flags,
/// the layout, and the subtree views.
struct RenderOptions {
    /// Node-path glob and collapse flags; when set, only matched nodes and their
    /// ancestors print.
    pattern: Option<PatternView>,

    /// The rendering to draw the populated grid through.
    layout: HierarchyShowLayout,

    /// Collapse repeat instances to a stub after the first placement.
    collapse_instances: bool,

    /// The per-node and per-object subtrees to append.
    views: HierarchyViews,
}

/// Renders the scene graph of `state` under `options`, the testable core of
/// [`hierarchy_show`]. Errors when a `pattern` is malformed or matches nothing.
fn render(state: &VoxMain, options: &RenderOptions) -> Result<String> {
    let scene = Scene::from_state(state);

    let filter = match &options.pattern {
        Some(pattern) => Some(scene.build_filter(pattern)?),
        None => None,
    };

    let mut walk = Walk {
        scene: &scene,
        collapse_instances: options.collapse_instances,
        views: options.views,
        filter,
        seen_nodes: IdVec::from_vec(vec![0; state.hierarchy_node_count()]),
        seen_objects: IdVec::from_vec(vec![0; state.object_count()]),
        grid: TreeGrid::new(),
    };

    // Collapsed ancestors print a flat list whose roots take connectors; the
    // section form prints `root` / `unplaced` as bare headers.
    let bare_roots = !walk.collapse_ancestors();

    walk.run();

    Ok(match options.layout {
        HierarchyShowLayout::Hierarchy => {
            let hierarchy = TreeGridHierarchyOptions::default().with_bare_roots(bare_roots);
            walk.grid.render_hierarchy(&hierarchy)
        }

        HierarchyShowLayout::JsonPretty => walk.grid.render_json_pretty(),

        HierarchyShowLayout::JsonCompact => walk.grid.render_json_compact(),
    })
}

/// A thin view over the loaded [`VoxMain`], which stays the single source of
/// truth for names, children, transforms, and grid boxes. The only derived data
/// is the placement count per node and object, tallied once and read back to
/// drive the instanced and unplaced marks: a node's count is its parent
/// references plus a root listing, an object's its parent references; two or
/// more is instanced, zero unplaced. The counts live in dense id-indexed
/// columns, sized once to the loaded state's live counts, since a loaded state
/// is never mutated here and numbers its ids `0..count`.
struct Scene<'a> {
    /// The loaded state, read directly for every per-entity lookup.
    state: &'a VoxMain,

    /// Placement count per node, indexed by node id.
    node_placements: IdVec<BVoxHierarchyNode, usize>,

    /// Placement count per object, indexed by object id.
    object_placements: IdVec<BVoxObject, usize>,
}

impl<'a> Scene<'a> {
    /// Tallies the placement counts over `state`; every other datum is read back
    /// from `state` on demand.
    fn from_state(state: &'a VoxMain) -> Scene<'a> {
        /// Tallies one placement for `id`, ignoring an id outside the sized
        /// range so an unvalidated document's dangling reference is a no-op
        /// rather than a panic.
        fn bump<TBrand: ?Sized>(counts: &mut IdVec<TBrand, usize>, id: U32Id<TBrand>) {
            if let Some(count) = counts.get_mut(id.to_usize_id()) {
                *count += 1;
            }
        }

        let mut node_placements: IdVec<BVoxHierarchyNode, usize> =
            IdVec::from_vec(vec![0; state.hierarchy_node_count()]);

        let mut object_placements: IdVec<BVoxObject, usize> =
            IdVec::from_vec(vec![0; state.object_count()]);

        for &root in state.root_hierarchy_nodes() {
            bump(&mut node_placements, root);
        }

        for (_, node) in state.iter_hierarchy_nodes() {
            for &child in &node.child_nodes {
                bump(&mut node_placements, child);
            }

            for &object in &node.child_objects {
                bump(&mut object_placements, object);
            }
        }

        Scene {
            state,
            node_placements,
            object_placements,
        }
    }

    /// The scene's roots, hierarchy node ids in listing order.
    fn roots(&self) -> &[NodeId] {
        self.state.root_hierarchy_nodes()
    }

    /// Placement count of node `id`, or `0` when `id` is outside the sized
    /// range.
    fn node_placement(&self, id: NodeId) -> usize {
        self.node_placements
            .get(id.to_usize_id())
            .copied()
            .unwrap_or(0)
    }

    /// Placement count of object `id`, or `0` when `id` is outside the sized
    /// range.
    fn object_placement(&self, id: ObjectId) -> usize {
        self.object_placements
            .get(id.to_usize_id())
            .copied()
            .unwrap_or(0)
    }

    /// Display name of node `id`, or `""` when it does not resolve.
    fn node_name(&self, id: NodeId) -> &str {
        self.state
            .hierarchy_node(id)
            .map(|node| node.name.as_str())
            .unwrap_or("")
    }

    /// Display name of object `id`, or `""` when it does not resolve.
    fn object_name(&self, id: ObjectId) -> &str {
        self.state
            .object(id)
            .map(|object| object.name())
            .unwrap_or("")
    }

    /// Node ids that are neither a root nor a child, in listing order.
    fn unplaced_nodes(&self) -> Vec<NodeId> {
        self.state
            .iter_hierarchy_nodes()
            .map(|(id, _)| id)
            .filter(|&id| self.node_placement(id) == 0)
            .collect()
    }

    /// Object ids that no node places, in listing order.
    fn orphan_objects(&self) -> Vec<ObjectId> {
        self.state
            .iter_objects()
            .map(|(id, _)| id)
            .filter(|&id| self.object_placement(id) == 0)
            .collect()
    }

    /// Every placement in the scene: each root subtree then each unplaced-node
    /// subtree, the node placements and the object placements beneath them, then
    /// the orphan objects no node places. Each carries its path, the chain of
    /// names from its section root, and the world transform of whatever places
    /// it. A node reached through several parents yields one placement per path.
    /// A node on its own ancestor chain is recorded once and not re-entered, so a
    /// cyclic document still terminates.
    fn enumerate_placements(&self) -> Vec<Placement> {
        let mut out = Vec::new();
        let mut branch = HashSet::new();
        let identity = TyTransformF64::default();

        for &root in self.roots() {
            let path = self.node_name(root).to_string();
            self.enumerate_from(root, path, identity, None, &mut branch, &mut out);
        }

        for id in self.unplaced_nodes() {
            let path = self.node_name(id).to_string();
            self.enumerate_from(id, path, identity, None, &mut branch, &mut out);
        }

        // An orphan object has just its name as a path and no placing node.
        for id in self.orphan_objects() {
            out.push(Placement {
                path: self.object_name(id).to_string(),
                entity: Entity::Object(id),
                parent_world: identity,
                parent: None,
            });
        }

        out
    }

    /// Records this node placement and its child objects, then descends into its
    /// child nodes unless `id` is already on the current branch, a cycle.
    fn enumerate_from(
        &self,
        id: NodeId,
        path: String,
        parent_world: TyTransformF64,
        parent: Option<usize>,
        branch: &mut HashSet<NodeId>,
        out: &mut Vec<Placement>,
    ) {
        let Some(node) = self.state.hierarchy_node(id) else {
            return;
        };

        let this = out.len();
        out.push(Placement {
            path: path.clone(),
            entity: Entity::Node(id),
            parent_world,
            parent,
        });

        if !branch.insert(id) {
            return;
        }

        let world = parent_world.compose(&node.transform);

        for &object in &node.child_objects {
            out.push(Placement {
                path: child_path(&path, self.object_name(object)),
                entity: Entity::Object(object),
                parent_world: world,
                parent: Some(this),
            });
        }

        for &child in &node.child_nodes {
            let child_path = child_path(&path, self.node_name(child));
            self.enumerate_from(child, child_path, world, Some(this), branch, out);
        }

        branch.remove(&id);
    }

    /// Builds the selection filter for `pattern`, erroring on a malformed
    /// pattern or when nothing matches.
    fn build_filter(&self, pattern: &PatternView) -> Result<Filter> {
        let patterns = GitIgnoreRegex::from_spans_ignore_inert(&pattern.globs)
            .map_err(|error| IOError::new(ErrorKind::InvalidInput, error.to_string()))?;

        let placements = self.enumerate_placements();

        let matched: Vec<bool> = placements
            .iter()
            .map(|placement| {
                let matched = if matches!(placement.entity, Entity::Node(_)) {
                    pathspec::is_directory_path_match(&patterns, &placement.path)
                } else {
                    pathspec::is_file_path_match(&patterns, &placement.path)
                };
                matched == Some(true)
            })
            .collect();

        if !matched.contains(&true) {
            return Err(IOError::new(
                ErrorKind::NotFound,
                format!(
                    "no node or object matched pattern '{}'",
                    pattern.globs.join("' '")
                ),
            )
            .into());
        }

        let parents: Vec<Option<usize>> = placements
            .iter()
            .map(|placement| placement.parent)
            .collect();
        let selection = TreeSelection::from_matches(matched, &parents);

        let selected = flagged_paths(&placements, selection.selected());
        let visible = flagged_paths(&placements, selection.visible());

        let roots: Vec<Placement> = selection
            .match_roots()
            .iter()
            .map(|&index| placements[index].clone())
            .collect();

        let root_paths: HashSet<String> = roots
            .iter()
            .map(|placement| placement.path.clone())
            .collect();

        Ok(Filter {
            selected,
            visible,
            roots,
            root_paths,
            collapse_ancestors: pattern.collapse_ancestors,
            collapse_descendants: pattern.collapse_descendants,
        })
    }
}

/// One placement from [`Scene::enumerate_placements`]: a node or an object, its
/// path, and the world transform of whatever places it.
#[derive(Clone)]
struct Placement {
    /// The chain of names from the section root to this placement.
    path: String,

    /// The node or object placed here.
    entity: Entity,

    /// The world transform of the placing parent.
    parent_world: TyTransformF64,

    /// The placing parent's index in the enumeration, `None` at a section
    /// root or an orphan object.
    parent: Option<usize>,
}

/// A placement's entity: a hierarchy node or a leaf object.
#[derive(Clone, Copy)]
enum Entity {
    /// A hierarchy node.
    Node(NodeId),

    /// A leaf object.
    Object(ObjectId),
}

/// Joins `parent` and `name` into a path, dropping an empty parent so a path
/// never leads with a separator.
fn child_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// Projects `flags` onto placement paths for the walk's path-keyed queries.
fn flagged_paths(placements: &[Placement], flags: &[bool]) -> HashSet<String> {
    placements
        .iter()
        .zip(flags)
        .filter(|&(_, &flag)| flag)
        .map(|(placement, _)| placement.path.clone())
        .collect()
}

/// A `pattern`'s resolved selection and its collapse flags. `selected` is every
/// path the patterns select, node and object alike; `visible` adds each selected
/// path's proper prefixes, the node paths that lead to a selection. `roots` and
/// `root_paths` are the selected placements whose parent is unselected, the entry
/// point of each selected subtree, for the collapse features. The collapse flags
/// live here because they act only with a pattern.
struct Filter {
    /// Every selected path, node and object alike.
    selected: HashSet<String>,

    /// Selected paths and every proper prefix, the node paths to show.
    visible: HashSet<String>,

    /// Match roots, in enumeration order, for the collapse features.
    roots: Vec<Placement>,

    /// Match-root paths, for an `O(1)` lookup while walking.
    root_paths: HashSet<String>,

    /// Hide each match's ancestor chain behind an `ancestors` marker.
    collapse_ancestors: bool,

    /// Hide each match's descendants behind a `descendants` marker.
    collapse_descendants: bool,
}

impl Filter {
    /// True if node `path` is selected or leads to a selection, so it shows.
    fn shows_node(&self, path: &str) -> bool {
        self.visible.contains(path)
    }

    /// True if object `path` is selected, so it shows.
    fn shows_object(&self, path: &str) -> bool {
        self.selected.contains(path)
    }

    /// True if `path` is a match root.
    fn is_root(&self, path: &str) -> bool {
        self.root_paths.contains(path)
    }
}

/// One populate pass over a [`Scene`].
struct Walk<'a> {
    /// The scene being rendered.
    scene: &'a Scene<'a>,

    /// Collapse repeat instances to a stub after the first placement.
    collapse_instances: bool,

    /// The per-node and per-object subtrees to append.
    views: HierarchyViews,

    /// The path filter, when a `pattern` was given.
    filter: Option<Filter>,

    /// Placements of each node already shown, indexed by node id, so each
    /// instance gets its index.
    seen_nodes: IdVec<BVoxHierarchyNode, usize>,

    /// Placements of each object already shown, indexed by object id.
    seen_objects: IdVec<BVoxObject, usize>,

    /// The grid being populated, rendered once the walk completes.
    grid: TreeGrid,
}

impl Walk<'_> {
    /// Whether the filter, when present, asks to collapse ancestors.
    fn collapse_ancestors(&self) -> bool {
        self.filter.as_ref().is_some_and(|f| f.collapse_ancestors)
    }

    /// Whether the filter, when present, asks to collapse descendants.
    fn collapse_descendants(&self) -> bool {
        self.filter.as_ref().is_some_and(|f| f.collapse_descendants)
    }

    /// The instance index of node `id`: how many of its placements have already
    /// been shown, recording this one.
    fn node_instance(&mut self, id: NodeId) -> usize {
        let slot = &mut self.seen_nodes[id.to_usize_id()];
        let index = *slot;
        *slot += 1;
        index
    }

    /// The instance index of object `id`: how many of its placements have
    /// already been shown, recording this one.
    fn object_instance(&mut self, id: ObjectId) -> usize {
        let slot = &mut self.seen_objects[id.to_usize_id()];
        let index = *slot;
        *slot += 1;
        index
    }

    /// Adds a node under `parent`, or as a grid root when there is none.
    fn add_node(&mut self, parent: Option<GridNodeId>, label: TreeGridLabel) -> GridNodeId {
        match parent {
            Some(parent) => self.grid.add_child(parent, label),
            None => self.grid.add_root(label),
        }
    }

    /// Adds a leaf under `parent` carrying one pre-formatted value.
    fn add_value_leaf(&mut self, parent: GridNodeId, label: impl Into<String>, text: String) {
        let leaf = self.grid.add_child(parent, TreeGridLabel::bare(label));
        self.grid.push_value(leaf, TreeGridValue::new(text));
    }

    /// Populates the whole scene into the grid.
    fn run(&mut self) {
        if self.collapse_ancestors() {
            self.run_collapsed_ancestors();
        } else {
            self.run_sections();
        }
    }

    /// The `root` section, then the `unplaced` section of unplaced nodes and
    /// orphan objects.
    fn run_sections(&mut self) {
        let roots = self.scene.roots().to_vec();
        self.build_group("root", &roots, &[]);

        let unplaced = self.scene.unplaced_nodes();
        let orphans = self.scene.orphan_objects();
        self.build_group("unplaced", &unplaced, &orphans);
    }

    /// Adds the match roots as a flat list, each behind an `ancestors` marker
    /// unless it is top-level. Each keeps its stored parent world transform, so
    /// the hidden ancestors' placement holds.
    fn run_collapsed_ancestors(&mut self) {
        let roots = match &self.filter {
            Some(filter) => filter.roots.clone(),
            None => return,
        };

        for placement in &roots {
            let parent = placement
                .path
                .contains('/')
                .then(|| self.grid.add_root(TreeGridLabel::bare("ancestors")));

            self.build_placement(placement, parent);
        }
    }

    /// Builds one match root: a node with its subtree, or a leaf object.
    fn build_placement(&mut self, placement: &Placement, parent: Option<GridNodeId>) {
        match placement.entity {
            Entity::Node(id) => {
                let mut branch = HashSet::new();
                self.build_node(
                    id,
                    parent,
                    &placement.path,
                    placement.parent_world,
                    &mut branch,
                );
            }

            Entity::Object(id) => self.build_object(id, parent, placement.parent_world),
        }
    }

    /// Whether the filter marks `path` as a match root.
    fn is_root(&self, path: &str) -> bool {
        self.filter.as_ref().is_some_and(|f| f.is_root(path))
    }

    /// Builds one section: a bare `header` root over the visible top-level
    /// nodes then objects, skipped entirely when empty.
    fn build_group(&mut self, header: &str, node_ids: &[NodeId], object_ids: &[ObjectId]) {
        let nodes: Vec<NodeId> = node_ids
            .iter()
            .copied()
            .filter(|&id| self.top_visible(id))
            .collect();

        let objects: Vec<ObjectId> = object_ids
            .iter()
            .copied()
            .filter(|&id| self.top_object_visible(id))
            .collect();

        if nodes.is_empty() && objects.is_empty() {
            return;
        }

        let section = self.grid.add_root(TreeGridLabel::bare(header));

        // A section root has no parent, so it starts from the identity.
        let identity = TyTransformF64::default();
        let mut branch = HashSet::new();

        for &id in &nodes {
            let path = self.scene.node_name(id).to_string();
            self.build_node(id, Some(section), &path, identity, &mut branch);
        }

        for &id in &objects {
            self.build_object(id, Some(section), identity);
        }
    }

    /// Whether a section-root node shows: always without a filter, else only when
    /// its own path is on the way to a selection.
    fn top_visible(&self, id: NodeId) -> bool {
        match &self.filter {
            None => true,
            Some(filter) => filter.shows_node(self.scene.node_name(id)),
        }
    }

    /// Whether a section-level object shows: always without a filter, else only
    /// when its name, its whole path in the section, is selected.
    fn top_object_visible(&self, id: ObjectId) -> bool {
        match &self.filter {
            None => true,
            Some(filter) => filter.shows_object(self.scene.object_name(id)),
        }
    }

    /// Builds node `id`'s subtree at `path`. A node on its own ancestor chain
    /// stops with a `cycle: true` tag, so a document that skipped validation
    /// cannot recurse forever.
    fn build_node(
        &mut self,
        id: NodeId,
        parent: Option<GridNodeId>,
        path: &str,
        parent_world: TyTransformF64,
        branch: &mut HashSet<NodeId>,
    ) {
        let scene = self.scene;

        let Some(node) = scene.state.hierarchy_node(id) else {
            self.add_node(
                parent,
                TreeGridLabel::bare(format!("missing node {}", id.to_u32())),
            );
            return;
        };

        let is_cycle = branch.contains(&id);

        // The instance index counts placements of this id already shown; it is
        // present only for a shared node, and a nonzero index is a repeat.
        let instance = (scene.node_placement(id) >= 2).then(|| self.node_instance(id));

        // Under collapse, a repeat placement outside a cycle is stubbed; the
        // nonzero instance index already marks it as a repeat.
        let collapsed_stub =
            self.collapse_instances && !is_cycle && instance.is_some_and(|index| index > 0);

        let mut tag = format!("node: {}", id.to_u32());

        if let Some(index) = instance {
            tag.push_str(&format!(", instance: {index}"));
        }

        if is_cycle {
            tag.push_str(", cycle: true");
        }

        let grid_node = self.add_node(parent, TreeGridLabel::quoted(node.name.as_str()));

        self.grid
            .push_value(grid_node, TreeGridValue::new(format!("{{{tag}}}")));

        if is_cycle || collapsed_stub {
            return;
        }

        let local = node.transform;

        let world = parent_world.compose(&local);

        if self.views.transforms.is_some() {
            self.build_transform(grid_node, local, world);
        }

        // The child nodes that lead to a selection and the child objects that are
        // selected; without a filter every child shows.
        let mut child_nodes: Vec<(NodeId, String)> = Vec::new();
        for &child in &node.child_nodes {
            let child_path = child_path(path, scene.node_name(child));
            if self
                .filter
                .as_ref()
                .is_none_or(|f| f.shows_node(&child_path))
            {
                child_nodes.push((child, child_path));
            }
        }

        let mut child_objects: Vec<ObjectId> = Vec::new();
        for &object in &node.child_objects {
            let object_path = child_path(path, scene.object_name(object));
            if self
                .filter
                .as_ref()
                .is_none_or(|f| f.shows_object(&object_path))
            {
                child_objects.push(object);
            }
        }

        let has_children = !child_nodes.is_empty() || !child_objects.is_empty();

        branch.insert(id);

        if self.collapse_descendants() && self.is_root(path) && has_children {
            self.grid
                .add_child(grid_node, TreeGridLabel::bare("descendants"));
        } else {
            for (child, child_path) in child_nodes {
                self.build_node(child, Some(grid_node), &child_path, world, branch);
            }

            for object in child_objects {
                self.build_object(object, Some(grid_node), world);
            }
        }

        branch.remove(&id);
    }

    /// Builds the `transform` subtree, in local or, when the view asks, the
    /// composed world space.
    fn build_transform(
        &mut self,
        parent: GridNodeId,
        local: TyTransformF64,
        world: TyTransformF64,
    ) {
        let Some(view) = self.views.transforms else {
            return;
        };

        let transform = if view.world { world } else { local };

        let subtree = self
            .grid
            .add_child(parent, TreeGridLabel::bare("transform"));

        let mut rotation = transform.rotation.to_euler_radians();

        if view.degrees {
            rotation *= 180.0 / PI;
        }

        let precision = view.precision;

        self.add_value_leaf(
            subtree,
            "position",
            format!("[{}]", format_vec3(transform.position, precision)),
        );

        self.add_value_leaf(
            subtree,
            "rotation",
            format!("[{}]", format_vec3(rotation, precision)),
        );

        self.add_value_leaf(
            subtree,
            "scale",
            format!("[{}]", format_vec3(transform.scale, precision)),
        );
    }

    /// Builds object `id`, its enabled rows, then its `layers` subtree.
    /// `placing_world` is the world transform of the placing node.
    fn build_object(
        &mut self,
        id: ObjectId,
        parent: Option<GridNodeId>,
        placing_world: TyTransformF64,
    ) {
        let scene = self.scene;

        let Some(object) = scene.state.object(id) else {
            self.add_node(
                parent,
                TreeGridLabel::bare(format!("missing object {}", id.to_u32())),
            );
            return;
        };

        let instance = (scene.object_placement(id) >= 2).then(|| self.object_instance(id));

        let mut tag = format!("object: {}", id.to_u32());

        if let Some(index) = instance {
            tag.push_str(&format!(", instance: {index}"));
        }

        let grid_node = self.add_node(parent, TreeGridLabel::quoted(object.name()));

        self.grid
            .push_value(grid_node, TreeGridValue::new(format!("{{{tag}}}")));

        for row in self.object_rows(object, placing_world) {
            self.build_object_row(&row, grid_node);
        }

        if self.views.layers {
            self.build_layers(grid_node, object);
        }
    }

    /// The enabled rows for `object`, in display order. An edit grid with no
    /// authoring margin yields its rows as `null`; the runtime grid is always
    /// shown, a zero-size box at the object's origin when it has no live voxels.
    /// `placing_world` folds an origin into world space when its view asks.
    fn object_rows(&self, object: &VoxObject, placing_world: TyTransformF64) -> Vec<ObjectRow> {
        let origin = object.origin().as_dvec3();
        let build = object.bounds().as_dvec3();
        let edit = edit_present(object);

        // The runtime grid as (node-relative min corner, size). An object with no
        // live voxels has a zero-size grid at its origin.
        let (runtime_min, runtime_size) = match object.live_extent() {
            Some((min, size)) => (origin + min.as_dvec3(), size.as_dvec3()),
            None => (origin, TyVector3F64::ZERO),
        };

        let views = self.views;
        let mut rows = Vec::new();

        if let Some(view) = views.edit_origins {
            let value = edit.then(|| origin_value(origin, view.world, placing_world));
            rows.push(ObjectRow::value("edit-origin", value, view.precision));
        }

        if let Some(precision) = views.edit_bounds {
            let corners = edit.then(|| (origin, origin + build));
            rows.push(ObjectRow::bounds("edit-bounds", corners, precision));
        }

        if let Some(precision) = views.edit_extents {
            rows.push(ObjectRow::value(
                "edit-extents",
                edit.then_some(build),
                precision,
            ));
        }

        if let Some(view) = views.runtime_origins {
            let value = Some(origin_value(runtime_min, view.world, placing_world));
            rows.push(ObjectRow::value("runtime-origin", value, view.precision));
        }

        if let Some(precision) = views.runtime_bounds {
            let corners = Some((runtime_min, runtime_min + runtime_size));
            rows.push(ObjectRow::bounds("runtime-bounds", corners, precision));
        }

        if let Some(precision) = views.runtime_extents {
            rows.push(ObjectRow::value(
                "runtime-extents",
                Some(runtime_size),
                precision,
            ));
        }

        if views.voxel_counts {
            rows.push(ObjectRow::count("voxel-count", object.live_count()));
        }

        rows
    }

    /// Builds one geometry row; an absent grid reads `null`.
    fn build_object_row(&mut self, row: &ObjectRow, parent: GridNodeId) {
        match row {
            ObjectRow::Value {
                label,
                value,
                precision,
            } => {
                let text = match value {
                    Some(vector) => format!("[{}]", format_vec3(*vector, *precision)),
                    None => "null".to_string(),
                };

                self.add_value_leaf(parent, *label, text);
            }

            ObjectRow::Count { label, value } => {
                self.add_value_leaf(parent, *label, value.to_string())
            }

            ObjectRow::Bounds {
                label,
                corners: None,
                ..
            } => self.add_value_leaf(parent, *label, "null".to_string()),

            ObjectRow::Bounds {
                label,
                corners: Some((min, max)),
                precision,
            } => {
                let subtree = self.grid.add_child(parent, TreeGridLabel::bare(*label));

                self.add_value_leaf(
                    subtree,
                    "min",
                    format!("[{}]", format_vec3(*min, *precision)),
                );

                self.add_value_leaf(
                    subtree,
                    "max",
                    format!("[{}]", format_vec3(*max, *precision)),
                );
            }
        }
    }

    /// Builds the `layers` subtree, one child per layer, `layers: []` when the
    /// object has none.
    fn build_layers(&mut self, parent: GridNodeId, object: &VoxObject) {
        if object.layer_count() == 0 {
            self.add_value_leaf(parent, "layers", "[]".to_string());
            return;
        }

        let subtree = self.grid.add_child(parent, TreeGridLabel::bare("layers"));

        for (_, palette_id) in object.iter_layers() {
            match self.scene.state.palette(palette_id) {
                Some(palette) => {
                    let materials = palette.material_count();
                    self.add_value_leaf(
                        subtree,
                        palette_id.to_u32().to_string(),
                        format!("{{materials: {materials}}}"),
                    );
                }

                None => {
                    self.grid.add_child(
                        subtree,
                        TreeGridLabel::bare(format!("missing palette {}", palette_id.to_u32())),
                    );
                }
            }
        }
    }
}

/// One appended row under an object: a single vector value, a min/max subtree,
/// or a scalar count. The vector and subtree rows each have a `null` form for
/// when the underlying grid is absent.
enum ObjectRow {
    /// A `label: [x, y, z]` line, or `label: null` when `value` is `None`.
    Value {
        label: &'static str,
        value: Option<TyVector3F64>,
        precision: usize,
    },

    /// A `label` subtree over a min and a max corner, or `label: null` when
    /// `corners` is `None`.
    Bounds {
        label: &'static str,
        corners: Option<(TyVector3F64, TyVector3F64)>,
        precision: usize,
    },

    /// A `label: <count>` line for a scalar count.
    Count { label: &'static str, value: usize },
}

impl ObjectRow {
    /// A single-line vector row.
    fn value(label: &'static str, value: Option<TyVector3F64>, precision: usize) -> ObjectRow {
        ObjectRow::Value {
            label,
            value,
            precision,
        }
    }

    /// A min/max subtree row.
    fn bounds(
        label: &'static str,
        corners: Option<(TyVector3F64, TyVector3F64)>,
        precision: usize,
    ) -> ObjectRow {
        ObjectRow::Bounds {
            label,
            corners,
            precision,
        }
    }

    /// A scalar count row.
    fn count(label: &'static str, value: usize) -> ObjectRow {
        ObjectRow::Count { label, value }
    }
}

/// Whether `object` has a distinct edit grid: its build volume adds margin around
/// the tight live extent or offsets it from the grid min corner. This mirrors the
/// condition `info` uses to report an object's edit bounds. An empty object counts
/// as having an edit grid when its build volume is non-empty.
fn edit_present(object: &VoxObject) -> bool {
    let build = object.bounds();

    match object.live_extent() {
        Some((min, size)) => {
            min.x != 0
                || min.y != 0
                || min.z != 0
                || size.x != build.x
                || size.y != build.y
                || size.z != build.z
        }

        None => build.x != 0 || build.y != 0 || build.z != 0,
    }
}

/// An origin corner in the space its view asks for: the node-local offset itself,
/// or that corner as a point through the placing node's `world` transform.
fn origin_value(corner: TyVector3F64, world: bool, placing_world: TyTransformF64) -> TyVector3F64 {
    if world {
        placing_world.transform_point(corner)
    } else {
        corner
    }
}

/// Formats a vector as `x, y, z`, each to `precision` decimal places.
fn format_vec3(vector: TyVector3F64, precision: usize) -> String {
    // Add zero to fold any -0.0 component to 0.0 so a zero never renders as
    // "-0.00" (a euler decomposition can yield a signed zero).
    let vector = vector + TyVector3F64::ZERO;
    format!(
        "{:.p$}, {:.p$}, {:.p$}",
        vector.x,
        vector.y,
        vector.z,
        p = precision
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        Result,
        commands::{HierarchyShowLayout, HierarchyViews, OriginView, PatternView, TransformView},
        implementation::hierarchy_show::{RenderOptions, render},
    };
    use branded_id::U32Id;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject,
        VoxPalette, VoxValuePool,
    };

    /// Renders `state` with the given pattern and collapse flags, unwrapping.
    fn show(
        state: &VoxMain,
        pattern: Option<&str>,
        collapse_instances: bool,
        collapse_ancestors: bool,
        collapse_descendants: bool,
    ) -> String {
        try_show(
            state,
            pattern,
            collapse_instances,
            collapse_ancestors,
            collapse_descendants,
        )
        .unwrap()
    }

    /// Renders `state`, returning the error instead of unwrapping.
    fn try_show(
        state: &VoxMain,
        pattern: Option<&str>,
        collapse_instances: bool,
        collapse_ancestors: bool,
        collapse_descendants: bool,
    ) -> Result<String> {
        let patterns: Vec<&str> = pattern.into_iter().collect();
        try_show_many(
            state,
            &patterns,
            collapse_instances,
            collapse_ancestors,
            collapse_descendants,
        )
    }

    /// Renders `state` with several patterns, unwrapping.
    fn show_many(
        state: &VoxMain,
        patterns: &[&str],
        collapse_instances: bool,
        collapse_ancestors: bool,
        collapse_descendants: bool,
    ) -> String {
        try_show_many(
            state,
            patterns,
            collapse_instances,
            collapse_ancestors,
            collapse_descendants,
        )
        .unwrap()
    }

    /// Renders `state` with several patterns, returning the error instead of
    /// unwrapping. An empty slice means no filter.
    fn try_show_many(
        state: &VoxMain,
        patterns: &[&str],
        collapse_instances: bool,
        collapse_ancestors: bool,
        collapse_descendants: bool,
    ) -> Result<String> {
        // The collapse flags live inside the pattern; without one they are moot,
        // which the command enforces with a clap `requires`.
        let pattern = (!patterns.is_empty()).then(|| PatternView {
            globs: patterns.iter().map(|glob| glob.to_string()).collect(),
            collapse_ancestors,
            collapse_descendants,
        });
        render(
            state,
            &RenderOptions {
                pattern,
                layout: HierarchyShowLayout::Hierarchy,
                collapse_instances,
                views: HierarchyViews::default(),
            },
        )
    }

    /// Renders `state` with the given views, no pattern or collapse flags.
    fn render_views(state: &VoxMain, views: HierarchyViews) -> String {
        render(
            state,
            &RenderOptions {
                pattern: None,
                layout: HierarchyShowLayout::Hierarchy,
                collapse_instances: false,
                views,
            },
        )
        .unwrap()
    }

    /// Renders `state` under `layout`, no pattern, collapse flags, or views.
    fn render_layout(state: &VoxMain, layout: HierarchyShowLayout) -> String {
        render(
            state,
            &RenderOptions {
                pattern: None,
                layout,
                collapse_instances: false,
                views: HierarchyViews::default(),
            },
        )
        .unwrap()
    }

    /// A 1x1x1 object; only its name matters to the tree.
    fn object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
    }

    /// An object with build volume `bounds`, grid `origin`, and every voxel in
    /// the inclusive `[lo, hi]` box made live, so its tight runtime extent is that
    /// box. `lo`/`hi` must lie inside `bounds`.
    fn object_live(
        name: &str,
        bounds: (u32, u32, u32),
        origin: (i32, i32, i32),
        lo: (u32, u32, u32),
        hi: (u32, u32, u32),
    ) -> VoxObject {
        let mut object = VoxObject::new(
            name.to_owned(),
            TyVector3U32::new(bounds.0, bounds.1, bounds.2),
        )
        .unwrap();
        object.set_origin(TyVector3I32::new(origin.0, origin.1, origin.2));
        for x in lo.0..=hi.0 {
            for y in lo.1..=hi.1 {
                for z in lo.2..=hi.2 {
                    let id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();
                    object.retain_voxel(id, &[]).unwrap();
                }
            }
        }
        object
    }

    /// A node carrying `transform`, placing the given child nodes and objects.
    fn node_xf(
        name: &str,
        transform: TyTransformF64,
        child_nodes: Vec<U32Id<BVoxHierarchyNode>>,
        child_objects: Vec<U32Id<BVoxObject>>,
    ) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_nodes,
            child_objects,
            transform,
        }
    }

    /// A transform from a position, a `z`-axis rotation in degrees, and a scale.
    fn xf(position: (f64, f64, f64), z_degrees: f64, scale: (f64, f64, f64)) -> TyTransformF64 {
        let rotation = TyQuaternionF64::from_axis_angle(
            TyVector3F64::new(0.0, 0.0, 1.0),
            z_degrees.to_radians(),
        );
        TyTransformF64::new(
            TyVector3F64::new(position.0, position.1, position.2),
            rotation,
            TyVector3F64::new(scale.0, scale.1, scale.2),
        )
    }

    /// A node with the given name, child nodes, and child objects.
    fn node(
        name: &str,
        child_nodes: Vec<U32Id<BVoxHierarchyNode>>,
        child_objects: Vec<U32Id<BVoxObject>>,
    ) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_nodes,
            child_objects,
            ..VoxHierarchyNode::default()
        }
    }

    /// A root placing one object under one node.
    fn simple_state() -> VoxMain {
        let mut state = VoxMain::default();
        let body = state.add_object(object("body")).unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();
        state
    }

    /// A root whose two arms both place one shared leaf node, which itself
    /// places one object: the leaf is instanced.
    fn instanced_state() -> VoxMain {
        let mut state = VoxMain::default();
        let head = state.add_object(object("head")).unwrap();
        let leaf = state
            .add_hierarchy_node(node("leaf", vec![], vec![head]))
            .unwrap();
        let arm_a = state
            .add_hierarchy_node(node("armA", vec![leaf], vec![]))
            .unwrap();
        let arm_b = state
            .add_hierarchy_node(node("armB", vec![leaf], vec![]))
            .unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![arm_a, arm_b], vec![]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();
        state
    }

    /// A root with two arms; `armA` places node `hand` (object `handMesh`) and
    /// `armB` places node `foot` (object `footMesh`). Node ids: hand 0, foot 1,
    /// armA 2, armB 3, root 4; object ids: handMesh 0, footMesh 1.
    fn pattern_state() -> VoxMain {
        let mut state = VoxMain::default();
        let hand_mesh = state.add_object(object("handMesh")).unwrap();
        let foot_mesh = state.add_object(object("footMesh")).unwrap();
        let hand = state
            .add_hierarchy_node(node("hand", vec![], vec![hand_mesh]))
            .unwrap();
        let foot = state
            .add_hierarchy_node(node("foot", vec![], vec![foot_mesh]))
            .unwrap();
        let arm_a = state
            .add_hierarchy_node(node("armA", vec![hand], vec![]))
            .unwrap();
        let arm_b = state
            .add_hierarchy_node(node("armB", vec![foot], vec![]))
            .unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![arm_a, arm_b], vec![]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();
        state
    }

    /// Adds a palette with `count` `baseColorFactor` materials to `state`; only
    /// the material count matters to the tree, so every color is the same.
    fn add_palette_with_materials(state: &mut VoxMain, count: usize) -> U32Id<BVoxPalette> {
        let pool = state.add_value_pool(VoxValuePool::srgba(vec![[0.0, 0.0, 0.0, 1.0]]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .add_property("baseColorFactor".to_owned(), pool, U32Id::from_u32(0))
            .unwrap();
        for _ in 0..count {
            palette.add_material(vec![U32Id::from_u32(0)]).unwrap();
        }
        state.add_palette(palette).unwrap()
    }

    /// A root placing one object `body` that carries a layer on palette 0 (two
    /// materials) then a layer on palette 1 (three materials).
    fn palette_ref_state() -> VoxMain {
        let mut state = VoxMain::default();

        let first = add_palette_with_materials(&mut state, 2);
        let first_material = state
            .palette(first)
            .unwrap()
            .iter_materials()
            .next()
            .unwrap();
        let second = add_palette_with_materials(&mut state, 3);
        let second_material = state
            .palette(second)
            .unwrap()
            .iter_materials()
            .next()
            .unwrap();

        let mut body = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        body.add_layer(first, first_material);
        body.add_layer(second, second_material);
        let body_id = state.add_object(body).unwrap();

        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body_id]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();
        state
    }

    /// A root placing one object `body` with a distinct edit grid: build volume
    /// 6x6x6 at origin (-1, -1, -1), live voxels tight in [1, 4] on each axis, so
    /// the runtime grid is a 4x4x4 box at node-relative origin (0, 0, 0).
    fn geometry_state() -> VoxMain {
        let mut state = VoxMain::default();
        let body = state
            .add_object(object_live(
                "body",
                (6, 6, 6),
                (-1, -1, -1),
                (1, 1, 1),
                (4, 4, 4),
            ))
            .unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();
        state
    }

    #[test]
    fn markdown_renders_a_simple_tree() {
        let output = show(&simple_state(), None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n"
        );
    }

    #[test]
    fn json_pretty_renders_the_record_envelope() {
        // The section root, the node tags, and the object tags all survive as
        // records; labels are the raw names, unquoted, and every value is the
        // pre-formatted tag text.
        let output = render_layout(&simple_state(), HierarchyShowLayout::JsonPretty);
        assert_eq!(
            output,
            r#"[
  {
    "label": "root",
    "children": [
      {
        "label": "root",
        "values": [
          "{node: 0}"
        ],
        "children": [
          {
            "label": "body",
            "values": [
              "{object: 0}"
            ]
          }
        ]
      }
    ]
  }
]
"#
        );
    }

    #[test]
    fn json_compact_renders_the_envelope_on_one_line() {
        let output = render_layout(&simple_state(), HierarchyShowLayout::JsonCompact);
        assert_eq!(
            output,
            concat!(
                r#"[{"label":"root","children":[{"label":"root","values":["{node: 0}"],"#,
                r#""children":[{"label":"body","values":["{object: 0}"]}]}]}]"#,
                "\n"
            )
        );
    }

    #[test]
    fn markdown_marks_every_instance_by_default() {
        let output = show(&instanced_state(), None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 3}\n\
             \u{20}\u{20}\u{251C} \"armA\": {node: 1}\n\
             \u{20}\u{20}\u{2502} \u{2514} \"leaf\": {node: 0, instance: 0}\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} \"head\": {object: 0}\n\
             \u{20}\u{20}\u{2514} \"armB\": {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"leaf\": {node: 0, instance: 1}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} \"head\": {object: 0}\n"
        );
    }

    #[test]
    fn collapse_instances_stubs_repeat_placements() {
        let output = show(&instanced_state(), None, true, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 3}\n\
             \u{20}\u{20}\u{251C} \"armA\": {node: 1}\n\
             \u{20}\u{20}\u{2502} \u{2514} \"leaf\": {node: 0, instance: 0}\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} \"head\": {object: 0}\n\
             \u{20}\u{20}\u{2514} \"armB\": {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"leaf\": {node: 0, instance: 1}\n"
        );
    }

    #[test]
    fn markdown_lists_unplaced_nodes_and_orphan_objects() {
        let mut state = VoxMain::default();
        let body = state.add_object(object("body")).unwrap();
        // `looseMesh` (object 1) is placed by no node: an orphan object.
        state.add_object(object("looseMesh")).unwrap();
        let spare_child = state.add_object(object("spareChild")).unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body]))
            .unwrap();
        // `spareNode` is neither a root nor a child: an unplaced library node.
        state
            .add_hierarchy_node(node("spareNode", vec![], vec![spare_child]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let output = show(&state, None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \n\
             unplaced\n\
             \u{251C} \"spareNode\": {node: 1}\n\
             \u{2502} \u{2514} \"spareChild\": {object: 2}\n\
             \u{2514} \"looseMesh\": {object: 1}\n"
        );
    }

    #[test]
    fn pattern_keeps_only_matches_and_their_ancestors() {
        // `**/hand` matches `root/armA/hand`; the `armB`/`foot` branch is pruned,
        // and the matched node's object shows.
        let output = show(&pattern_state(), Some("hand"), false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 4}\n\
             \u{20}\u{20}\u{2514} \"armA\": {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} \"handMesh\": {object: 0}\n"
        );
    }

    #[test]
    fn a_pattern_matching_nothing_is_an_error() {
        let result = try_show(&pattern_state(), Some("missing"), false, false, false);
        assert!(result.is_err());
    }

    #[test]
    fn multiple_patterns_union_their_matches() {
        // `hand` and `foot` each select a branch; both show, objects included.
        let output = show_many(&pattern_state(), &["hand", "foot"], false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 4}\n\
             \u{20}\u{20}\u{251C} \"armA\": {node: 2}\n\
             \u{20}\u{20}\u{2502} \u{2514} \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} \"handMesh\": {object: 0}\n\
             \u{20}\u{20}\u{2514} \"armB\": {node: 3}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"foot\": {node: 1}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} \"footMesh\": {object: 1}\n"
        );
    }

    #[test]
    fn a_bang_pattern_deselects_a_subtree() {
        // `**` selects everything, then `!armB/` prunes the armB branch, leaving
        // only the armA branch. Git-faithful: the prune is final.
        let output = show_many(&pattern_state(), &["**", "!armB/"], false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 4}\n\
             \u{20}\u{20}\u{2514} \"armA\": {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} \"handMesh\": {object: 0}\n"
        );
    }

    #[test]
    fn an_object_pattern_selects_the_object_and_shows_its_ancestors() {
        // `**/footMesh` selects only that object; its node chain shows as context
        // even though no node matched.
        let output = show_many(&pattern_state(), &["**/footMesh"], false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 4}\n\
             \u{20}\u{20}\u{2514} \"armB\": {node: 3}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"foot\": {node: 1}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} \"footMesh\": {object: 1}\n"
        );
    }

    #[test]
    fn an_orphan_object_is_selectable_by_name() {
        let mut state = VoxMain::default();
        let body = state.add_object(object("body")).unwrap();
        // `looseMesh` (object 1) is placed by no node: an orphan object.
        state.add_object(object("looseMesh")).unwrap();
        let spare_child = state.add_object(object("spareChild")).unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body]))
            .unwrap();
        state
            .add_hierarchy_node(node("spareNode", vec![], vec![spare_child]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        // The orphan object matches; nothing else does, so only it shows.
        let output = show_many(&state, &["looseMesh"], false, false, false);
        assert_eq!(
            output,
            "unplaced\n\
             \u{2514} \"looseMesh\": {object: 1}\n"
        );
    }

    #[test]
    fn collapse_ancestors_hides_the_chain_above_a_match() {
        // The ancestor chain `root/armA` is replaced by one marker; no section
        // header, since the match is shown flat.
        let output = show(&pattern_state(), Some("hand"), false, true, false);
        assert_eq!(
            output,
            "\u{2514} ancestors\n\
             \u{20}\u{20}\u{2514} \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"handMesh\": {object: 0}\n"
        );
    }

    #[test]
    fn collapse_descendants_hides_the_subtree_below_a_match() {
        let output = show(&pattern_state(), Some("hand"), false, false, true);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 4}\n\
             \u{20}\u{20}\u{2514} \"armA\": {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} descendants\n"
        );
    }

    #[test]
    fn collapse_ancestors_and_descendants_combine() {
        let output = show(&pattern_state(), Some("hand"), false, true, true);
        assert_eq!(
            output,
            "\u{2514} ancestors\n\
             \u{20}\u{20}\u{2514} \"hand\": {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} descendants\n"
        );
    }

    #[test]
    fn collapse_ancestors_prints_an_empty_named_child_once() {
        // Empty names collapse the object's path onto its parent node's, so
        // match-rootness must come from the parent link, not the path.
        let mut state = VoxMain::default();
        let body = state.add_object(object("")).unwrap();
        let root = state
            .add_hierarchy_node(node("", vec![], vec![body]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let output = show(&state, Some("*"), false, true, false);
        assert_eq!(
            output,
            "\u{2514} \"\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"\": {object: 0}\n"
        );
    }

    #[test]
    fn collapse_flags_do_nothing_without_a_pattern() {
        let plain = show(&pattern_state(), None, false, false, false);
        let with_flags = show(&pattern_state(), None, false, true, true);
        assert_eq!(plain, with_flags);
    }

    #[test]
    fn transforms_local_prepend_the_node_transform() {
        let mut state = VoxMain::default();
        let body = state.add_object(object("body")).unwrap();
        let root = state
            .add_hierarchy_node(node_xf(
                "root",
                xf((1.0, 2.0, 3.0), 90.0, (1.0, 1.0, 1.0)),
                vec![],
                vec![body],
            ))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let view = TransformView {
            world: false,
            degrees: true,
            precision: 2,
        };
        let output = render_views(
            &state,
            HierarchyViews {
                transforms: Some(view),
                ..HierarchyViews::default()
            },
        );
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{251C} transform\n\
             \u{20}\u{20}\u{2502} \u{251C} position: [1.00, 2.00, 3.00]\n\
             \u{20}\u{20}\u{2502} \u{251C} rotation: [0.00, 0.00, 90.00]\n\
             \u{20}\u{20}\u{2502} \u{2514} scale: [1.00, 1.00, 1.00]\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n"
        );
    }

    #[test]
    fn edit_and_runtime_geometry_render_all_six_rows() {
        // Every geometry flag on, at precision 2 in local space. Edit is the
        // 6x6x6 build volume at origin (-1, -1, -1); runtime is the tight 4x4x4
        // live box, node-relative origin (0, 0, 0).
        let views = HierarchyViews {
            edit_origins: Some(OriginView {
                world: false,
                precision: 2,
            }),
            edit_bounds: Some(2),
            edit_extents: Some(2),
            runtime_origins: Some(OriginView {
                world: false,
                precision: 2,
            }),
            runtime_bounds: Some(2),
            runtime_extents: Some(2),
            ..HierarchyViews::default()
        };
        let output = render_views(&geometry_state(), views);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} edit-origin: [-1.00, -1.00, -1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} edit-bounds\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{251C} min: [-1.00, -1.00, -1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{2514} max: [5.00, 5.00, 5.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} edit-extents: [6.00, 6.00, 6.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} runtime-origin: [0.00, 0.00, 0.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} runtime-bounds\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{251C} min: [0.00, 0.00, 0.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{2514} max: [4.00, 4.00, 4.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} runtime-extents: [4.00, 4.00, 4.00]\n"
        );
    }

    #[test]
    fn edit_geometry_is_null_without_an_authoring_margin() {
        // A 2x2x2 object fully live from the corner: build volume equals the tight
        // extent, so it has no distinct edit grid and every edit row is `null`.
        let mut state = VoxMain::default();
        let body = state
            .add_object(object_live(
                "body",
                (2, 2, 2),
                (0, 0, 0),
                (0, 0, 0),
                (1, 1, 1),
            ))
            .unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let views = HierarchyViews {
            edit_origins: Some(OriginView {
                world: false,
                precision: 2,
            }),
            edit_bounds: Some(2),
            edit_extents: Some(2),
            runtime_extents: Some(2),
            ..HierarchyViews::default()
        };
        let output = render_views(&state, views);
        assert!(
            output.contains("edit-origin: null"),
            "output was:\n{output}"
        );
        assert!(
            output.contains("edit-bounds: null"),
            "output was:\n{output}"
        );
        assert!(
            output.contains("edit-extents: null"),
            "output was:\n{output}"
        );
        assert!(
            output.contains("runtime-extents: [2.00, 2.00, 2.00]"),
            "output was:\n{output}"
        );
    }

    #[test]
    fn runtime_geometry_is_a_zero_box_at_the_origin_for_an_empty_object() {
        // A 3x3x3 build volume at origin (-1, -1, -1) with no live voxels: the
        // runtime grid is a zero-size box at that origin, never `null`, while the
        // edit rows read the build volume.
        let mut state = VoxMain::default();
        let mut body = VoxObject::new("body".to_owned(), TyVector3U32::new(3, 3, 3)).unwrap();
        body.set_origin(TyVector3I32::new(-1, -1, -1));
        let body = state.add_object(body).unwrap();
        let root = state
            .add_hierarchy_node(node("root", vec![], vec![body]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let views = HierarchyViews {
            edit_extents: Some(2),
            runtime_origins: Some(OriginView {
                world: false,
                precision: 2,
            }),
            runtime_bounds: Some(2),
            runtime_extents: Some(2),
            ..HierarchyViews::default()
        };
        let output = render_views(&state, views);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} edit-extents: [3.00, 3.00, 3.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} runtime-origin: [-1.00, -1.00, -1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} runtime-bounds\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{251C} min: [-1.00, -1.00, -1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{2514} max: [-1.00, -1.00, -1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} runtime-extents: [0.00, 0.00, 0.00]\n"
        );
    }

    #[test]
    fn runtime_origins_world_apply_the_node_transform() {
        // A unit object fully live at the grid corner under a node translated to
        // +10x. Its runtime origin is (0, 0, 0) locally, (10, 0, 0) in world.
        let mut state = VoxMain::default();
        let body = state
            .add_object(object_live(
                "body",
                (1, 1, 1),
                (0, 0, 0),
                (0, 0, 0),
                (0, 0, 0),
            ))
            .unwrap();
        let root = state
            .add_hierarchy_node(node_xf(
                "root",
                xf((10.0, 0.0, 0.0), 0.0, (1.0, 1.0, 1.0)),
                vec![],
                vec![body],
            ))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let local = render_views(
            &state,
            HierarchyViews {
                runtime_origins: Some(OriginView {
                    world: false,
                    precision: 2,
                }),
                ..HierarchyViews::default()
            },
        );
        assert!(
            local.contains("runtime-origin: [0.00, 0.00, 0.00]"),
            "output was:\n{local}"
        );

        let world = render_views(
            &state,
            HierarchyViews {
                runtime_origins: Some(OriginView {
                    world: true,
                    precision: 2,
                }),
                ..HierarchyViews::default()
            },
        );
        assert!(
            world.contains("runtime-origin: [10.00, 0.00, 0.00]"),
            "output was:\n{world}"
        );
    }

    #[test]
    fn transforms_world_composes_the_parent_chain() {
        // A child at local +1x under a parent translated to +10x sits at world
        // +11x.
        let mut state = VoxMain::default();
        let body = state.add_object(object("body")).unwrap();
        let child = state
            .add_hierarchy_node(node_xf(
                "child",
                xf((1.0, 0.0, 0.0), 0.0, (1.0, 1.0, 1.0)),
                vec![],
                vec![body],
            ))
            .unwrap();
        let root = state
            .add_hierarchy_node(node_xf(
                "root",
                xf((10.0, 0.0, 0.0), 0.0, (1.0, 1.0, 1.0)),
                vec![child],
                vec![],
            ))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let view = TransformView {
            world: true,
            degrees: false,
            precision: 2,
        };
        let output = render_views(
            &state,
            HierarchyViews {
                transforms: Some(view),
                ..HierarchyViews::default()
            },
        );
        assert!(
            output.contains("position: [11.00, 0.00, 0.00]"),
            "output was:\n{output}"
        );
    }

    #[test]
    fn layers_list_each_referenced_palette_with_its_material_count() {
        let output = render_views(
            &palette_ref_state(),
            HierarchyViews {
                layers: true,
                ..HierarchyViews::default()
            },
        );
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} layers\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{251C} 0: {materials: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} 1: {materials: 3}\n"
        );
    }

    #[test]
    fn layers_are_an_empty_array_when_an_object_has_none() {
        // `simple_state`'s `body` has no layer, so the subtree collapses to an
        // empty array rather than a childless header.
        let output = render_views(
            &simple_state(),
            HierarchyViews {
                layers: true,
                ..HierarchyViews::default()
            },
        );
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} layers: []\n"
        );
    }

    #[test]
    fn layers_follow_the_geometry_rows_under_an_object() {
        // With a geometry row and layers both on, layers is the last child, so
        // the geometry row keeps its non-last connector.
        let output = render_views(
            &palette_ref_state(),
            HierarchyViews {
                edit_extents: Some(2),
                layers: true,
                ..HierarchyViews::default()
            },
        );
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} edit-extents: [1.00, 1.00, 1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} layers\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{251C} 0: {materials: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} 1: {materials: 3}\n"
        );
    }

    #[test]
    fn voxel_counts_report_the_filled_voxel_count() {
        // `geometry_state`'s body fills the tight [1, 4] box on each axis, a
        // 4x4x4 block of 64 live voxels.
        let output = render_views(
            &geometry_state(),
            HierarchyViews {
                voxel_counts: true,
                ..HierarchyViews::default()
            },
        );
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} voxel-count: 64\n"
        );
    }

    #[test]
    fn voxel_counts_precede_the_layers_subtree() {
        // With the voxel count and layers both on, the count is a plain leaf
        // above the layers subtree, matching info's voxels-then-layers order.
        // `body` holds no live voxel, so the count is `0`.
        let output = render_views(
            &palette_ref_state(),
            HierarchyViews {
                voxel_counts: true,
                layers: true,
                ..HierarchyViews::default()
            },
        );
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"root\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"body\": {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} voxel-count: 0\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} layers\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{251C} 0: {materials: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} 1: {materials: 3}\n"
        );
    }

    #[test]
    fn names_are_quoted_so_empty_and_spaced_names_stay_legible() {
        // A nameless node and an object whose name carries a space both print
        // quoted; the `root` section header stays unquoted.
        let mut state = VoxMain::default();
        let mesh = state.add_object(object("my mesh")).unwrap();
        let root = state
            .add_hierarchy_node(node("", vec![], vec![mesh]))
            .unwrap();
        state.set_root_hierarchy_nodes(vec![root]).unwrap();

        let output = show(&state, None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} \"\": {node: 0}\n\
             \u{20}\u{20}\u{2514} \"my mesh\": {object: 0}\n"
        );
    }
}
