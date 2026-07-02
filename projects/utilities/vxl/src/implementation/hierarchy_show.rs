use crate::{BoundsView, Format, PatternView, Result, TransformView, implementation};
use branded_id::{IdVec, U32Id};
use pathspec::GitIgnoreRegex;
use std::{
    collections::HashSet,
    f64::consts::PI,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use ty_math::{TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxMain, VoxObject};

/// A hierarchy-node id in the loaded [`VoxMain`], aliased so signatures stay
/// short and a node id never mixes with an object id.
type NodeId = U32Id<BVoxHierarchyNode>;

/// An object id in the loaded [`VoxMain`], aliased alongside [`NodeId`] to keep
/// the node-versus-object distinction in the type system.
type ObjectId = U32Id<BVoxObject>;

/// Box drawings up and right: the connector before a last child.
const CONNECTOR_LAST: char = '\u{2514}';

/// Box drawings vertical and right: the connector before a non-last child.
const CONNECTOR_MID: char = '\u{251C}';

/// The prefix extension under a last child: two spaces.
const EXTENSION_LAST: &str = "  ";

/// The prefix extension under a non-last child: box drawings vertical, a space.
const EXTENSION_MID: &str = "\u{2502} ";

/// Loads the voxel file at `input` and prints its scene graph as a tree.
#[allow(clippy::too_many_arguments)]
pub fn hierarchy_show(
    input: &Path,
    from: Option<Format>,
    pattern: Option<PatternView>,
    collapse_instances: bool,
    transforms: Option<TransformView>,
    bounds: Option<BoundsView>,
    extents: Option<BoundsView>,
    palettes: bool,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let options = RenderOptions {
        pattern,
        collapse_instances,
        transforms,
        bounds,
        extents,
        palettes,
    };

    let output = render(&state, &options)?;

    implementation::write_stdout(output.as_bytes())
}

/// The knobs `render` reads: an optional node-path glob with its collapse flags,
/// and the subtree views.
struct RenderOptions {
    /// Node-path glob and collapse flags; when set, only matched nodes and their
    /// ancestors print.
    pattern: Option<PatternView>,

    /// Collapse repeat instances to a stub after the first placement.
    collapse_instances: bool,

    /// When set, prepend each node's transform as a subtree.
    transforms: Option<TransformView>,

    /// When set, append each object's grid bounds as a subtree.
    bounds: Option<BoundsView>,

    /// When set, append each object's extents as a subtree.
    extents: Option<BoundsView>,

    /// When true, append each object's referenced palettes as a subtree.
    palettes: bool,
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
        transforms: options.transforms,
        bounds: options.bounds,
        extents: options.extents,
        palettes: options.palettes,
        filter,
        seen_nodes: IdVec::from_vec(vec![0; state.hierarchy_node_count()]),
        seen_objects: IdVec::from_vec(vec![0; state.object_count()]),
        output: String::new(),
    };

    walk.run();

    Ok(walk.output)
}

/// One object's grid box: its build-volume size and its origin, the offset from
/// the placing node to the box's min corner, both in voxels.
#[derive(Clone, Copy)]
struct ObjectBox {
    /// Grid size in voxels.
    bounds: TyVector3U32,

    /// Offset from the placing node to the min corner, in voxels.
    origin: TyVector3I32,
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
            self.enumerate_from(root, path, identity, &mut branch, &mut out);
        }

        for id in self.unplaced_nodes() {
            let path = self.node_name(id).to_string();
            self.enumerate_from(id, path, identity, &mut branch, &mut out);
        }

        // An orphan object has just its name as a path and no placing node.
        for id in self.orphan_objects() {
            out.push(Placement {
                path: self.object_name(id).to_string(),
                entity: Entity::Object(id),
                parent_world: identity,
            });
        }

        out
    }

    /// Records this node placement and its child objects, then descends into its
    /// child nodes unless `id` is already on the current branch, a cycle.
    /// `parent_world` is the world transform of `id`'s parent; `branch` is the
    /// set of node ids on the path from the section root to here.
    fn enumerate_from(
        &self,
        id: NodeId,
        path: String,
        parent_world: TyTransformF64,
        branch: &mut HashSet<NodeId>,
        out: &mut Vec<Placement>,
    ) {
        let Some(node) = self.state.hierarchy_node(id) else {
            return;
        };

        out.push(Placement {
            path: path.clone(),
            entity: Entity::Node(id),
            parent_world,
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
            });
        }

        for &child in &node.child_nodes {
            let child_path = child_path(&path, self.node_name(child));
            self.enumerate_from(child, child_path, world, branch, out);
        }

        branch.remove(&id);
    }

    /// Builds the selection filter for `pattern`: every placement its patterns
    /// select, the node paths that lead to a selection, and the match roots, plus
    /// the collapse flags. Errors on a malformed pattern or when nothing matches.
    fn build_filter(&self, pattern: &PatternView) -> Result<Filter> {
        let patterns = GitIgnoreRegex::from_spans_ignore_inert(&pattern.globs)
            .map_err(|error| IOError::new(ErrorKind::InvalidInput, error.to_string()))?;

        let placements = self.enumerate_placements();

        let selected: HashSet<String> = placements
            .iter()
            .filter(|placement| {
                let is_dir = matches!(placement.entity, Entity::Node(_));
                pathspec::is_path_match(&patterns, &placement.path, is_dir) == Some(true)
            })
            .map(|placement| placement.path.clone())
            .collect();

        if selected.is_empty() {
            return Err(IOError::new(
                ErrorKind::NotFound,
                format!(
                    "no node or object matched pattern '{}'",
                    pattern.globs.join("' '")
                ),
            )
            .into());
        }

        // Every selected path and each of its proper prefixes, so the chain from
        // a section root down to a selection stays on screen.
        let mut visible = selected.clone();
        for path in &selected {
            for (index, _) in path.match_indices('/') {
                visible.insert(path[..index].to_string());
            }
        }

        // The selected placements whose parent is not itself selected: the entry
        // point of each selected subtree.
        let roots: Vec<Placement> = placements
            .into_iter()
            .filter(|placement| selected.contains(&placement.path))
            .filter(|placement| match parent_path(&placement.path) {
                Some(parent) => !selected.contains(parent),
                None => true,
            })
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

/// The parent path of `path`, or `None` when `path` is at the top level.
fn parent_path(path: &str) -> Option<&str> {
    path.rfind('/').map(|index| &path[..index])
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

/// One render pass over a [`Scene`]: the immutable options and filter, plus the
/// growing output and the per-id counters that number instances and drive the
/// instance collapse.
struct Walk<'a> {
    /// The scene being rendered.
    scene: &'a Scene<'a>,

    /// Collapse repeat instances to a stub after the first placement.
    collapse_instances: bool,

    /// When set, prepend each node's transform as a subtree.
    transforms: Option<TransformView>,

    /// When set, append each object's grid bounds as a subtree.
    bounds: Option<BoundsView>,

    /// When set, append each object's extents as a subtree.
    extents: Option<BoundsView>,

    /// When true, append each object's referenced palettes as a subtree.
    palettes: bool,

    /// The path filter, when a `pattern` was given.
    filter: Option<Filter>,

    /// Placements of each node already shown, indexed by node id, so each
    /// instance gets its index.
    seen_nodes: IdVec<BVoxHierarchyNode, usize>,

    /// Placements of each object already shown, indexed by object id.
    seen_objects: IdVec<BVoxObject, usize>,

    /// The accumulated tree text.
    output: String,
}

/// One ordered child of a node in the render: a prepended transform subtree, a
/// collapsed-descendants marker, a child node with its path, or a child object.
enum NodeChild {
    /// The node's own transform subtree.
    Transform,

    /// A `descendants` marker standing in for the hidden subtree.
    Descendants,

    /// A child node and its path.
    Node(NodeId, String),

    /// A child object.
    Object(ObjectId),
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

    /// Renders the whole scene into `output`.
    fn run(&mut self) {
        if self.collapse_ancestors() {
            self.run_collapsed_ancestors();
        } else {
            self.run_sections();
        }
    }

    /// The `root` section of each root's subtree, then the `unplaced` section of
    /// nodes that are neither a root nor a child and objects no node places. With
    /// a filter, each section keeps only entries on the way to a match, and a
    /// section header prints only when its section has visible entries.
    fn run_sections(&mut self) {
        let roots = self.scene.roots().to_vec();
        self.render_group("root", &roots, &[], false);

        let unplaced = self.scene.unplaced_nodes();
        let orphans = self.scene.orphan_objects();
        self.render_group("unplaced", &unplaced, &orphans, true);
    }

    /// Prints the match roots as a flat list, each behind an `ancestors` marker,
    /// dropped when the root is a top-level node. Runs only with a filter. World
    /// space still uses each root's stored parent world transform, so the hidden
    /// ancestors' placement is kept.
    fn run_collapsed_ancestors(&mut self) {
        let roots = match &self.filter {
            Some(filter) => filter.roots.clone(),
            None => return,
        };

        let total = roots.len();

        for (index, placement) in roots.iter().enumerate() {
            let is_last = index + 1 == total;
            let mut branch = HashSet::new();

            if placement.path.contains('/') {
                let connector = if is_last {
                    CONNECTOR_LAST
                } else {
                    CONNECTOR_MID
                };

                self.output.push_str(&format!("{connector} ancestors\n"));

                let extension = if is_last {
                    EXTENSION_LAST
                } else {
                    EXTENSION_MID
                };

                self.render_placement(placement, extension, true, &mut branch);
            } else {
                self.render_placement(placement, "", is_last, &mut branch);
            }
        }
    }

    /// Renders one match root: a node with its subtree, or a leaf object.
    fn render_placement(
        &mut self,
        placement: &Placement,
        prefix: &str,
        is_last: bool,
        branch: &mut HashSet<NodeId>,
    ) {
        match placement.entity {
            Entity::Node(id) => self.render_node(
                id,
                prefix,
                is_last,
                &placement.path,
                placement.parent_world,
                branch,
            ),

            Entity::Object(id) => self.render_object(id, prefix, is_last, placement.parent_world),
        }
    }

    /// Whether the filter marks `path` as a match root.
    fn is_root(&self, path: &str) -> bool {
        self.filter.as_ref().is_some_and(|f| f.is_root(path))
    }

    /// Renders one section under `header`: its top-level node ids then object
    /// ids, at prefix depth zero. With a filter, a top-level node shows only when
    /// it is on the way to a selection, and a section-level object shows only when
    /// its name is selected. Nothing prints, header included, when the section is
    /// empty.
    fn render_group(
        &mut self,
        header: &str,
        node_ids: &[NodeId],
        object_ids: &[ObjectId],
        gap: bool,
    ) {
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

        let total = nodes.len() + objects.len();

        if total == 0 {
            return;
        }

        if gap && !self.output.is_empty() {
            self.output.push('\n');
        }

        self.output.push_str(header);
        self.output.push('\n');

        // A section root has no parent, so it starts from the identity.
        let identity = TyTransformF64::default();
        let mut branch = HashSet::new();

        for (index, &id) in nodes.iter().enumerate() {
            let path = self.scene.node_name(id).to_string();
            self.render_node(id, "", index + 1 == total, &path, identity, &mut branch);
        }

        for (index, &id) in objects.iter().enumerate() {
            let last = nodes.len() + index + 1 == total;
            self.render_object(id, "", last, identity);
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

    /// Appends node `id`'s subtree at `path`. Every line reads
    /// `name: {node: <id>, ...}`: a shared node adds `instance: <k>`, the count
    /// of its placements already shown, so `instance > 0` marks a repeat, and
    /// with `collapse_instances` a repeat outside a cycle stops without
    /// expanding. A node on its own ancestor chain adds `cycle: true` and stops,
    /// so a document that skipped validation cannot recurse forever.
    /// `parent_world` is this node's parent's world transform,
    /// composed with the node's local transform for `--show-transforms world` and
    /// carried down to the children. With a filter, a child node shows when it
    /// leads to a selection and a child object shows when it is selected;
    /// `collapse_descendants` replaces a match root's subtree with a
    /// `descendants` marker.
    fn render_node(
        &mut self,
        id: NodeId,
        prefix: &str,
        is_last: bool,
        path: &str,
        parent_world: TyTransformF64,
        branch: &mut HashSet<NodeId>,
    ) {
        let scene = self.scene;

        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };

        let Some(node) = scene.state.hierarchy_node(id) else {
            self.output.push_str(&format!(
                "{prefix}{connector} missing node {}\n",
                id.to_u32()
            ));
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

        self.output
            .push_str(&format!("{prefix}{connector} {}: {{{tag}}}\n", node.name));

        if is_cycle || collapsed_stub {
            return;
        }

        let local = node.transform;

        let world = parent_world.compose(&local);

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };

        let child_prefix = format!("{prefix}{extension}");

        // Assemble the ordered children: the transform subtree first, then either
        // the collapsed-descendants marker or the filtered real children.
        let mut children: Vec<NodeChild> = Vec::new();

        if self.transforms.is_some() {
            children.push(NodeChild::Transform);
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

        if self.collapse_descendants() && self.is_root(path) && has_children {
            children.push(NodeChild::Descendants);
        } else {
            for (child, child_path) in child_nodes {
                children.push(NodeChild::Node(child, child_path));
            }

            for object in child_objects {
                children.push(NodeChild::Object(object));
            }
        }

        let total = children.len();

        branch.insert(id);

        for (index, child) in children.into_iter().enumerate() {
            let last = index + 1 == total;

            match child {
                NodeChild::Transform => self.render_transform(&child_prefix, last, local, world),

                NodeChild::Descendants => {
                    let connector = if last { CONNECTOR_LAST } else { CONNECTOR_MID };
                    self.output
                        .push_str(&format!("{child_prefix}{connector} descendants\n"));
                }

                NodeChild::Node(child_id, child_path) => {
                    self.render_node(child_id, &child_prefix, last, &child_path, world, branch)
                }

                NodeChild::Object(object) => self.render_object(object, &child_prefix, last, world),
            }
        }

        branch.remove(&id);
    }

    /// Appends the `transform` subtree for a node: its position, rotation, and
    /// scale, in local space or, when the view asks, the composed world space.
    fn render_transform(
        &mut self,
        prefix: &str,
        is_last: bool,
        local: TyTransformF64,
        world: TyTransformF64,
    ) {
        let Some(view) = self.transforms else {
            return;
        };

        let transform = if view.world { world } else { local };

        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };

        self.output
            .push_str(&format!("{prefix}{connector} transform\n"));

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };

        let inner = format!("{prefix}{extension}");

        let mut rotation = transform.rotation.to_euler_radians();

        if view.degrees {
            rotation = rotation * (180.0 / PI);
        }

        let precision = view.precision;

        self.output.push_str(&format!(
            "{inner}{CONNECTOR_MID} position: [{}]\n",
            format_vec3(transform.position, precision)
        ));

        self.output.push_str(&format!(
            "{inner}{CONNECTOR_MID} rotation: [{}]\n",
            format_vec3(rotation, precision)
        ));

        self.output.push_str(&format!(
            "{inner}{CONNECTOR_LAST} scale: [{}]\n",
            format_vec3(transform.scale, precision)
        ));
    }

    /// Appends object `id` as a leaf line reading `name: {object: <id>, ...}`,
    /// then its `bounds` and `extents` subtrees when set.
    /// `placing_world` is the world transform of the node placing the object.
    fn render_object(
        &mut self,
        id: ObjectId,
        prefix: &str,
        is_last: bool,
        placing_world: TyTransformF64,
    ) {
        let scene = self.scene;

        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };

        let Some(object) = scene.state.object(id) else {
            self.output.push_str(&format!(
                "{prefix}{connector} missing object {}\n",
                id.to_u32()
            ));
            return;
        };

        let instance = (scene.object_placement(id) >= 2).then(|| self.object_instance(id));

        let mut tag = format!("object: {}", id.to_u32());

        if let Some(index) = instance {
            tag.push_str(&format!(", instance: {index}"));
        }

        self.output.push_str(&format!(
            "{prefix}{connector} {}: {{{tag}}}\n",
            object.name()
        ));

        let object_box = ObjectBox {
            bounds: object.bounds(),
            origin: object.origin(),
        };

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };

        let child_prefix = format!("{prefix}{extension}");
        let bounds = self.bounds;
        let extents = self.extents;
        let palettes = self.palettes;

        if let Some(view) = bounds {
            self.render_bounds(
                &child_prefix,
                extents.is_none() && !palettes,
                object_box,
                placing_world,
                view,
            );
        }

        if let Some(view) = extents {
            self.render_extents(&child_prefix, !palettes, object_box, placing_world, view);
        }

        if palettes {
            self.render_palettes(&child_prefix, true, object);
        }
    }

    /// Appends the `bounds` subtree: the object's grid box as a min and a max
    /// corner, in local space or, when the view asks, world space.
    fn render_bounds(
        &mut self,
        prefix: &str,
        is_last: bool,
        object_box: ObjectBox,
        world: TyTransformF64,
        view: BoundsView,
    ) {
        let (min, max) = box_min_max(object_box, world, view.world);

        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };

        self.output
            .push_str(&format!("{prefix}{connector} bounds\n"));

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };

        let inner = format!("{prefix}{extension}");

        self.output.push_str(&format!(
            "{inner}{CONNECTOR_MID} min: [{}]\n",
            format_vec3(min, view.precision)
        ));

        self.output.push_str(&format!(
            "{inner}{CONNECTOR_LAST} max: [{}]\n",
            format_vec3(max, view.precision)
        ));
    }

    /// Appends the `extents` line: the object's grid box size, `max - min`, in
    /// local space or, when the view asks, world space.
    fn render_extents(
        &mut self,
        prefix: &str,
        is_last: bool,
        object_box: ObjectBox,
        world: TyTransformF64,
        view: BoundsView,
    ) {
        let (min, max) = box_min_max(object_box, world, view.world);

        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };

        self.output.push_str(&format!(
            "{prefix}{connector} extents: [{}]\n",
            format_vec3(max - min, view.precision)
        ));
    }

    /// Appends the `palettes` subtree: one child per palette the object
    /// references, in reference order, each `index: {cells: <count>}`. An object
    /// that references no palette prints `palettes: []`, and a reference to a
    /// palette the state does not hold prints a `missing palette` marker.
    fn render_palettes(&mut self, prefix: &str, is_last: bool, object: &VoxObject) {
        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };

        let total = object.palette_ref_count();

        if total == 0 {
            self.output
                .push_str(&format!("{prefix}{connector} palettes: []\n"));
            return;
        }

        self.output
            .push_str(&format!("{prefix}{connector} palettes\n"));

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };

        let inner = format!("{prefix}{extension}");

        for (index, (_, palette_id)) in object.iter_palette_refs().enumerate() {
            let child_connector = if index + 1 == total {
                CONNECTOR_LAST
            } else {
                CONNECTOR_MID
            };

            let line = match self.scene.state.palette(palette_id) {
                Some(palette) => format!(
                    "{inner}{child_connector} {}: {{cells: {}}}\n",
                    palette_id.to_u32(),
                    palette.cell_count()
                ),

                None => format!(
                    "{inner}{child_connector} missing palette {}\n",
                    palette_id.to_u32()
                ),
            };

            self.output.push_str(&line);
        }
    }
}

/// The min and max corners of `object_box`, in world space when `world_space`,
/// else in the node-local grid space. World space applies the placing node's
/// `world` transform to the box: its center moves as a point and its half-extents
/// grow by the absolute rotation, so the result is the box's axis-aligned bound.
fn box_min_max(
    object_box: ObjectBox,
    world: TyTransformF64,
    world_space: bool,
) -> (TyVector3F64, TyVector3F64) {
    let origin = object_box.origin;

    let bounds = object_box.bounds;

    let min = TyVector3F64::new(origin.x as f64, origin.y as f64, origin.z as f64);

    let size = TyVector3F64::new(bounds.x as f64, bounds.y as f64, bounds.z as f64);

    if !world_space {
        return (min, min + size);
    }

    let half = size * 0.5;

    let center = world.transform_point(min + half);

    let world_half = world
        .rotation
        .rotate_extents_abs(world.scale.abs().componentwise_multiply(&half));

    (center - world_half, center + world_half)
}

/// Formats a vector as `x, y, z`, each to `precision` decimal places.
fn format_vec3(vector: TyVector3F64, precision: usize) -> String {
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
        BoundsView, PatternView, Result, TransformView,
        implementation::hierarchy_show::{RenderOptions, render},
    };
    use branded_id::U32Id;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValue,
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
                collapse_instances,
                transforms: None,
                bounds: None,
                extents: None,
                palettes: false,
            },
        )
    }

    /// Renders `state` with the subtree views, no pattern or collapse flags.
    fn views(
        state: &VoxMain,
        transforms: Option<TransformView>,
        bounds: Option<BoundsView>,
        extents: Option<BoundsView>,
    ) -> String {
        render(
            state,
            &RenderOptions {
                pattern: None,
                collapse_instances: false,
                transforms,
                bounds,
                extents,
                palettes: false,
            },
        )
        .unwrap()
    }

    /// Renders `state` with the palettes subtree, no pattern or other views.
    fn palettes(state: &VoxMain) -> String {
        render(
            state,
            &RenderOptions {
                pattern: None,
                collapse_instances: false,
                transforms: None,
                bounds: None,
                extents: None,
                palettes: true,
            },
        )
        .unwrap()
    }

    /// A 1x1x1 object; only its name matters to the tree.
    fn object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
    }

    /// An object with a grid `bounds` size and an `origin` offset.
    fn object_box(name: &str, bounds: (u32, u32, u32), origin: (i32, i32, i32)) -> VoxObject {
        let mut object = VoxObject::new(
            name.to_owned(),
            TyVector3U32::new(bounds.0, bounds.1, bounds.2),
        )
        .unwrap();
        object.set_origin(TyVector3I32::new(origin.0, origin.1, origin.2));
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
        let body = state.add_object(object("body"));
        let root = state.add_hierarchy_node(node("root", vec![], vec![body]));
        state.set_root_hierarchy_nodes(vec![root]);
        state
    }

    /// A root whose two arms both place one shared leaf node, which itself
    /// places one object: the leaf is instanced.
    fn instanced_state() -> VoxMain {
        let mut state = VoxMain::default();
        let head = state.add_object(object("head"));
        let leaf = state.add_hierarchy_node(node("leaf", vec![], vec![head]));
        let arm_a = state.add_hierarchy_node(node("armA", vec![leaf], vec![]));
        let arm_b = state.add_hierarchy_node(node("armB", vec![leaf], vec![]));
        let root = state.add_hierarchy_node(node("root", vec![arm_a, arm_b], vec![]));
        state.set_root_hierarchy_nodes(vec![root]);
        state
    }

    /// A root with two arms; `armA` places node `hand` (object `handMesh`) and
    /// `armB` places node `foot` (object `footMesh`). Node ids: hand 0, foot 1,
    /// armA 2, armB 3, root 4; object ids: handMesh 0, footMesh 1.
    fn pattern_state() -> VoxMain {
        let mut state = VoxMain::default();
        let hand_mesh = state.add_object(object("handMesh"));
        let foot_mesh = state.add_object(object("footMesh"));
        let hand = state.add_hierarchy_node(node("hand", vec![], vec![hand_mesh]));
        let foot = state.add_hierarchy_node(node("foot", vec![], vec![foot_mesh]));
        let arm_a = state.add_hierarchy_node(node("armA", vec![hand], vec![]));
        let arm_b = state.add_hierarchy_node(node("armB", vec![foot], vec![]));
        let root = state.add_hierarchy_node(node("root", vec![arm_a, arm_b], vec![]));
        state.set_root_hierarchy_nodes(vec![root]);
        state
    }

    /// A palette with `count` `rgba` cells; only the cell count matters to the
    /// tree, so every color is the same.
    fn palette_with_cells(count: usize) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for _ in 0..count {
            palette
                .add_cell(vec![VoxValue::Text("#000000FF".to_owned())])
                .unwrap();
        }
        palette
    }

    /// A root placing one object `body` that references palette 0 (two cells)
    /// then palette 1 (three cells).
    fn palette_ref_state() -> VoxMain {
        let mut state = VoxMain::default();

        let first = state.add_palette(palette_with_cells(2));
        let first_cell = state.palette(first).unwrap().iter_cells().next().unwrap();
        let second = state.add_palette(palette_with_cells(3));
        let second_cell = state.palette(second).unwrap().iter_cells().next().unwrap();

        let mut body = VoxObject::new("body".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        body.add_palette_ref(first, first_cell);
        body.add_palette_ref(second, second_cell);
        let body_id = state.add_object(body);

        let root = state.add_hierarchy_node(node("root", vec![], vec![body_id]));
        state.set_root_hierarchy_nodes(vec![root]);
        state
    }

    #[test]
    fn markdown_renders_a_simple_tree() {
        let output = show(&simple_state(), None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n"
        );
    }

    #[test]
    fn markdown_marks_every_instance_by_default() {
        let output = show(&instanced_state(), None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 3}\n\
             \u{20}\u{20}\u{251C} armA: {node: 1}\n\
             \u{20}\u{20}\u{2502} \u{2514} leaf: {node: 0, instance: 0}\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} head: {object: 0}\n\
             \u{20}\u{20}\u{2514} armB: {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} leaf: {node: 0, instance: 1}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} head: {object: 0}\n"
        );
    }

    #[test]
    fn collapse_instances_stubs_repeat_placements() {
        let output = show(&instanced_state(), None, true, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 3}\n\
             \u{20}\u{20}\u{251C} armA: {node: 1}\n\
             \u{20}\u{20}\u{2502} \u{2514} leaf: {node: 0, instance: 0}\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} head: {object: 0}\n\
             \u{20}\u{20}\u{2514} armB: {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} leaf: {node: 0, instance: 1}\n"
        );
    }

    #[test]
    fn markdown_lists_unplaced_nodes_and_orphan_objects() {
        let mut state = VoxMain::default();
        let body = state.add_object(object("body"));
        // `looseMesh` (object 1) is placed by no node: an orphan object.
        state.add_object(object("looseMesh"));
        let spare_child = state.add_object(object("spareChild"));
        let root = state.add_hierarchy_node(node("root", vec![], vec![body]));
        // `spareNode` is neither a root nor a child: an unplaced library node.
        state.add_hierarchy_node(node("spareNode", vec![], vec![spare_child]));
        state.set_root_hierarchy_nodes(vec![root]);

        let output = show(&state, None, false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n\
             \n\
             unplaced\n\
             \u{251C} spareNode: {node: 1}\n\
             \u{2502} \u{2514} spareChild: {object: 2}\n\
             \u{2514} looseMesh: {object: 1}\n"
        );
    }

    #[test]
    fn a_reference_cycle_is_marked_and_not_re_entered() {
        // Two nodes each listing the other: a cycle a loader accepts, since
        // validation is a separate step. Ids are assigned in add order, so `a`
        // is 0 and `b` is 1; each references the other by that id.
        let mut state = VoxMain::default();
        let a = state.add_hierarchy_node(node("a", vec![U32Id::from_u32(1)], vec![]));
        state.add_hierarchy_node(node("b", vec![U32Id::from_u32(0)], vec![]));
        state.set_root_hierarchy_nodes(vec![a]);

        let output = show(&state, None, false, false, false);
        // `a` opens, `b` under it, then `a` again as a cycle leaf; it stops
        // there rather than recursing forever.
        assert!(output.contains("cycle: true"), "output was:\n{output}");
        assert_eq!(output.matches("{node: ").count(), 3);
    }

    #[test]
    fn pattern_keeps_only_matches_and_their_ancestors() {
        // `**/hand` matches `root/armA/hand`; the `armB`/`foot` branch is pruned,
        // and the matched node's object shows.
        let output = show(&pattern_state(), Some("hand"), false, false, false);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 4}\n\
             \u{20}\u{20}\u{2514} armA: {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} hand: {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} handMesh: {object: 0}\n"
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
             \u{2514} root: {node: 4}\n\
             \u{20}\u{20}\u{251C} armA: {node: 2}\n\
             \u{20}\u{20}\u{2502} \u{2514} hand: {node: 0}\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} handMesh: {object: 0}\n\
             \u{20}\u{20}\u{2514} armB: {node: 3}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} foot: {node: 1}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} footMesh: {object: 1}\n"
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
             \u{2514} root: {node: 4}\n\
             \u{20}\u{20}\u{2514} armA: {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} hand: {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} handMesh: {object: 0}\n"
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
             \u{2514} root: {node: 4}\n\
             \u{20}\u{20}\u{2514} armB: {node: 3}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} foot: {node: 1}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} footMesh: {object: 1}\n"
        );
    }

    #[test]
    fn an_orphan_object_is_selectable_by_name() {
        let mut state = VoxMain::default();
        let body = state.add_object(object("body"));
        // `looseMesh` (object 1) is placed by no node: an orphan object.
        state.add_object(object("looseMesh"));
        let spare_child = state.add_object(object("spareChild"));
        let root = state.add_hierarchy_node(node("root", vec![], vec![body]));
        state.add_hierarchy_node(node("spareNode", vec![], vec![spare_child]));
        state.set_root_hierarchy_nodes(vec![root]);

        // The orphan object matches; nothing else does, so only it shows.
        let output = show_many(&state, &["looseMesh"], false, false, false);
        assert_eq!(
            output,
            "unplaced\n\
             \u{2514} looseMesh: {object: 1}\n"
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
             \u{20}\u{20}\u{2514} hand: {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} handMesh: {object: 0}\n"
        );
    }

    #[test]
    fn collapse_descendants_hides_the_subtree_below_a_match() {
        let output = show(&pattern_state(), Some("hand"), false, false, true);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 4}\n\
             \u{20}\u{20}\u{2514} armA: {node: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} hand: {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} descendants\n"
        );
    }

    #[test]
    fn collapse_ancestors_and_descendants_combine() {
        let output = show(&pattern_state(), Some("hand"), false, true, true);
        assert_eq!(
            output,
            "\u{2514} ancestors\n\
             \u{20}\u{20}\u{2514} hand: {node: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} descendants\n"
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
        let body = state.add_object(object("body"));
        let root = state.add_hierarchy_node(node_xf(
            "root",
            xf((1.0, 2.0, 3.0), 90.0, (1.0, 1.0, 1.0)),
            vec![],
            vec![body],
        ));
        state.set_root_hierarchy_nodes(vec![root]);

        let view = TransformView {
            world: false,
            degrees: true,
            precision: 2,
        };
        let output = views(&state, Some(view), None, None);
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{251C} transform\n\
             \u{20}\u{20}\u{2502} \u{251C} position: [1.00, 2.00, 3.00]\n\
             \u{20}\u{20}\u{2502} \u{251C} rotation: [0.00, 0.00, 90.00]\n\
             \u{20}\u{20}\u{2502} \u{2514} scale: [1.00, 1.00, 1.00]\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n"
        );
    }

    #[test]
    fn bounds_and_extents_local_append_the_grid_box() {
        let mut state = VoxMain::default();
        let body = state.add_object(object_box("body", (4, 5, 6), (1, 0, 0)));
        let root = state.add_hierarchy_node(node("root", vec![], vec![body]));
        state.set_root_hierarchy_nodes(vec![root]);

        let view = BoundsView {
            world: false,
            precision: 2,
        };
        let output = views(&state, None, Some(view), Some(view));
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} bounds\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{251C} min: [1.00, 0.00, 0.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{2514} max: [5.00, 5.00, 6.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} extents: [4.00, 5.00, 6.00]\n"
        );
    }

    #[test]
    fn bounds_world_applies_the_placing_node_transform() {
        // A 2x1x1 box under a node turned 180 degrees about z. The exact
        // half-turn quaternion keeps the world corners integer.
        let mut state = VoxMain::default();
        let body = state.add_object(object_box("body", (2, 1, 1), (0, 0, 0)));
        let transform = TyTransformF64::new(
            TyVector3F64::new(0.0, 0.0, 0.0),
            TyQuaternionF64::new(0.0, 0.0, 1.0, 0.0),
            TyVector3F64::new(1.0, 1.0, 1.0),
        );
        let root = state.add_hierarchy_node(node_xf("root", transform, vec![], vec![body]));
        state.set_root_hierarchy_nodes(vec![root]);

        let view = BoundsView {
            world: true,
            precision: 2,
        };
        let output = views(&state, None, Some(view), Some(view));
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} bounds\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{251C} min: [-2.00, -1.00, 0.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{2514} max: [0.00, 0.00, 1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} extents: [2.00, 1.00, 1.00]\n"
        );
    }

    #[test]
    fn transforms_world_composes_the_parent_chain() {
        // A child at local +1x under a parent translated to +10x sits at world
        // +11x.
        let mut state = VoxMain::default();
        let body = state.add_object(object("body"));
        let child = state.add_hierarchy_node(node_xf(
            "child",
            xf((1.0, 0.0, 0.0), 0.0, (1.0, 1.0, 1.0)),
            vec![],
            vec![body],
        ));
        let root = state.add_hierarchy_node(node_xf(
            "root",
            xf((10.0, 0.0, 0.0), 0.0, (1.0, 1.0, 1.0)),
            vec![child],
            vec![],
        ));
        state.set_root_hierarchy_nodes(vec![root]);

        let view = TransformView {
            world: true,
            degrees: false,
            precision: 2,
        };
        let output = views(&state, Some(view), None, None);
        assert!(
            output.contains("position: [11.00, 0.00, 0.00]"),
            "output was:\n{output}"
        );
    }

    #[test]
    fn palettes_list_each_referenced_palette_with_its_cell_count() {
        let output = palettes(&palette_ref_state());
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} palettes\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{251C} 0: {cells: 2}\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} 1: {cells: 3}\n"
        );
    }

    #[test]
    fn palettes_are_an_empty_array_when_an_object_references_none() {
        // `simple_state`'s `body` has no palette reference, so the subtree
        // collapses to an empty array rather than a childless header.
        let output = palettes(&simple_state());
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} palettes: []\n"
        );
    }

    #[test]
    fn palettes_follow_bounds_and_extents_under_an_object() {
        // With all three object subtrees on, palettes is last, so bounds and
        // extents keep their non-last connectors.
        let mut state = VoxMain::default();
        let palette = state.add_palette(palette_with_cells(1));
        let cell = state.palette(palette).unwrap().iter_cells().next().unwrap();
        let mut body = VoxObject::new("body".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        body.add_palette_ref(palette, cell);
        let body_id = state.add_object(body);
        let root = state.add_hierarchy_node(node("root", vec![], vec![body_id]));
        state.set_root_hierarchy_nodes(vec![root]);

        let view = BoundsView {
            world: false,
            precision: 2,
        };
        let output = render(
            &state,
            &RenderOptions {
                pattern: None,
                collapse_instances: false,
                transforms: None,
                bounds: Some(view),
                extents: Some(view),
                palettes: true,
            },
        )
        .unwrap();
        assert_eq!(
            output,
            "root\n\
             \u{2514} root: {node: 0}\n\
             \u{20}\u{20}\u{2514} body: {object: 0}\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} bounds\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{251C} min: [0.00, 0.00, 0.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2502} \u{2514} max: [2.00, 1.00, 1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{251C} extents: [2.00, 1.00, 1.00]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} palettes\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} 0: {cells: 1}\n"
        );
    }
}
