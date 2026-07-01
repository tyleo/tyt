use crate::{BoundsView, Format, PathGlob, PatternView, Result, TransformView, implementation};
use branded_id::{IdVec, U32Id};
use std::{
    collections::HashSet,
    f64::consts::PI,
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use ty_math::{TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxMain};

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
pub fn hierarchy_show(
    input: &Path,
    from: Option<Format>,
    pattern: Option<PatternView>,
    collapse_instances: bool,
    transforms: Option<TransformView>,
    bounds: Option<BoundsView>,
    extents: Option<BoundsView>,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let options = RenderOptions {
        pattern,
        collapse_instances,
        transforms,
        bounds,
        extents,
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

    /// Every node placement: the roots' subtrees then the unplaced nodes'
    /// subtrees, each carrying its path (the chain of node names from its section
    /// root) and its parent's world transform. A node reached through several
    /// parents yields one entry per placement. A node on its own ancestor chain
    /// is recorded once and not re-entered, so a cyclic document still
    /// terminates.
    fn enumerate_node_paths(&self) -> Vec<NodePath> {
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

        out
    }

    /// Records this placement, then descends into its child nodes unless `id` is
    /// already on the current branch (a cycle). `parent_world` is the world
    /// transform of `id`'s parent; `branch` is the set of node ids on the path
    /// from the section root to here.
    fn enumerate_from(
        &self,
        id: NodeId,
        path: String,
        parent_world: TyTransformF64,
        branch: &mut HashSet<NodeId>,
        out: &mut Vec<NodePath>,
    ) {
        let Some(node) = self.state.hierarchy_node(id) else {
            return;
        };

        out.push(NodePath {
            path: path.clone(),
            id,
            parent_world,
        });

        if !branch.insert(id) {
            return;
        }

        let world = parent_world.compose(&node.transform);
        for &child in &node.child_nodes {
            let child_path = child_path(&path, self.node_name(child));
            self.enumerate_from(child, child_path, world, branch, out);
        }

        branch.remove(&id);
    }

    /// Builds the path filter for `pattern`: the node paths its glob matches and
    /// every proper prefix of a match, plus the pattern's collapse flags. The
    /// glob is normalized with a leading `**/` through [`PathGlob`]. Errors on a
    /// malformed glob or when nothing matches.
    fn build_filter(&self, pattern: &PatternView) -> Result<Filter> {
        let glob: PathGlob = pattern
            .glob
            .parse()
            .map_err(|message| IOError::new(ErrorKind::InvalidInput, message))?;

        let paths = self.enumerate_node_paths();

        let flags = {
            let candidates: Vec<&str> = paths.iter().map(|node| node.path.as_str()).collect();
            implementation::match_glob(glob.pattern(), &candidates)?
        };

        let matches: Vec<NodePath> = paths
            .into_iter()
            .zip(&flags)
            .filter_map(|(node, &matched)| matched.then_some(node))
            .collect();

        if matches.is_empty() {
            return Err(IOError::new(
                ErrorKind::NotFound,
                format!("no node matched pattern '{}'", pattern.glob),
            )
            .into());
        }

        let matched: HashSet<String> = matches.iter().map(|node| node.path.clone()).collect();

        let mut ancestors = HashSet::new();

        for node in &matches {
            let parts: Vec<&str> = node.path.split('/').collect();

            for end in 1..parts.len() {
                ancestors.insert(parts[..end].join("/"));
            }
        }

        Ok(Filter {
            matches,
            matched,
            ancestors,
            collapse_ancestors: pattern.collapse_ancestors,
            collapse_descendants: pattern.collapse_descendants,
        })
    }
}

/// One node placement from [`Scene::enumerate_node_paths`]: its path, its node
/// id, and its parent's world transform.
#[derive(Clone)]
struct NodePath {
    /// The chain of node names from the section root to this node.
    path: String,

    /// The node id.
    id: NodeId,

    /// The world transform of this node's parent.
    parent_world: TyTransformF64,
}

/// Joins `parent` and `name` into a node path, dropping an empty parent so a
/// path never leads with a separator.
fn child_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

/// A `pattern`'s resolved matches and its collapse flags. `matches` is every
/// matched placement, in enumeration order; `matched` and `ancestors` are the
/// matched paths and their proper prefixes, for `O(1)` lookups while walking. The
/// collapse flags live here because they act only with a pattern.
struct Filter {
    /// Matched placements, in enumeration order.
    matches: Vec<NodePath>,

    /// The matched node paths.
    matched: HashSet<String>,

    /// Every proper prefix of a matched path.
    ancestors: HashSet<String>,

    /// Hide each match's ancestor chain behind an `ancestors` marker.
    collapse_ancestors: bool,

    /// Hide each match's descendants behind a `descendants` marker.
    collapse_descendants: bool,
}

impl Filter {
    /// True if `path` is a match or leads to one, so its subtree stays visible.
    fn on_path(&self, path: &str) -> bool {
        self.matched.contains(path) || self.ancestors.contains(path)
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

    /// Prints matches as a flat list, each match's subtree behind an
    /// `ancestors` marker, dropped when the match is a section root. Runs only
    /// with a filter, so the ancestor chain being hidden is well defined. World
    /// space still uses the match's stored parent world transform, so the hidden
    /// ancestors' placement is kept.
    fn run_collapsed_ancestors(&mut self) {
        let matches = match &self.filter {
            Some(filter) => filter.matches.clone(),
            None => return,
        };

        let total = matches.len();
        for (index, node) in matches.iter().enumerate() {
            let is_last = index + 1 == total;
            let mut branch = HashSet::new();

            if node.path.contains('/') {
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

                self.render_node(
                    node.id,
                    extension,
                    true,
                    &node.path,
                    true,
                    node.parent_world,
                    &mut branch,
                );
            } else {
                self.render_node(
                    node.id,
                    "",
                    is_last,
                    &node.path,
                    true,
                    node.parent_world,
                    &mut branch,
                );
            }
        }
    }

    /// Renders one section under `header`: its top-level node ids then object
    /// ids, at prefix depth zero. With a filter, a top-level node shows only when
    /// it is on the way to a match, and orphan objects, which no node path names,
    /// are dropped. Nothing prints, header included, when the section is empty.
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

        let objects: Vec<ObjectId> = if self.filter.is_some() {
            Vec::new()
        } else {
            object_ids.to_vec()
        };

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
            self.render_node(
                id,
                "",
                index + 1 == total,
                &path,
                false,
                identity,
                &mut branch,
            );
        }

        for (index, &id) in objects.iter().enumerate() {
            let last = nodes.len() + index + 1 == total;
            self.render_object(id, "", last, identity);
        }
    }

    /// Whether a section-root node shows: always without a filter, else only when
    /// its own path is on the way to a match.
    fn top_visible(&self, id: NodeId) -> bool {
        match &self.filter {
            None => true,
            Some(filter) => filter.on_path(self.scene.node_name(id)),
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
    /// carried down to the children. `in_match` is set once this or an ancestor
    /// matched: below a match the whole subtree shows, but `collapse_descendants`
    /// replaces it with a `descendants` marker; above a match a filter keeps
    /// only the child nodes leading to one.
    #[allow(clippy::too_many_arguments)]
    fn render_node(
        &mut self,
        id: NodeId,
        prefix: &str,
        is_last: bool,
        path: &str,
        in_match: bool,
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

        let matched = self
            .filter
            .as_ref()
            .is_some_and(|f| f.matched.contains(path));

        let in_match = in_match || matched;

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

        let has_children = !node.child_nodes.is_empty() || !node.child_objects.is_empty();

        if self.collapse_descendants() && in_match && has_children {
            children.push(NodeChild::Descendants);
        } else {
            // In a match, or with no filter, show every child; above a match keep
            // only the child nodes leading to one, dropping objects, which no
            // node path names.
            let show_all = self.filter.is_none() || in_match;

            for &child in &node.child_nodes {
                let child_path = child_path(path, scene.node_name(child));
                if show_all || self.filter.as_ref().is_some_and(|f| f.on_path(&child_path)) {
                    children.push(NodeChild::Node(child, child_path));
                }
            }

            if show_all {
                for &object in &node.child_objects {
                    children.push(NodeChild::Object(object));
                }
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

                NodeChild::Node(child_id, child_path) => self.render_node(
                    child_id,
                    &child_prefix,
                    last,
                    &child_path,
                    in_match,
                    world,
                    branch,
                ),

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

        if let Some(view) = bounds {
            self.render_bounds(
                &child_prefix,
                extents.is_none(),
                object_box,
                placing_world,
                view,
            );
        }

        if let Some(view) = extents {
            self.render_extents(&child_prefix, true, object_box, placing_world, view);
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
    use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};

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
        // The collapse flags live inside the pattern; without one they are moot,
        // which the command enforces with a clap `requires`.
        let pattern = pattern.map(|glob| PatternView {
            glob: glob.to_owned(),
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
}
