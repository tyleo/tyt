use crate::{Format, PathGlob, Result, implementation};
use std::{
    collections::{HashMap, HashSet},
    io::{Error as IOError, ErrorKind},
    path::Path,
};
use voxcore::VoxMain;

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
    pattern: Option<String>,
    collapse_instances: bool,
    collapse_ancestors: bool,
    collapse_descendants: bool,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let options = RenderOptions {
        pattern,
        collapse_instances,
        collapse_ancestors,
        collapse_descendants,
    };
    let output = render(&state, &options)?;
    implementation::write_stdout(output.as_bytes())
}

/// The knobs `render` reads: an optional node-path glob and the three collapse
/// flags. The collapse flags act only alongside a `pattern`.
struct RenderOptions {
    /// Node-path glob; when set, only matched nodes and their ancestors print.
    pattern: Option<String>,
    /// Collapse repeat instances to a stub after the first placement.
    collapse_instances: bool,
    /// Hide each match's ancestor chain behind an `[Ancestors]` marker.
    collapse_ancestors: bool,
    /// Hide each match's descendants behind a `[Descendants]` marker.
    collapse_descendants: bool,
}

/// Renders the scene graph of `state` under `options`, the testable core of
/// [`hierarchy_show`]. Errors when a `pattern` is malformed or matches nothing.
fn render(state: &VoxMain, options: &RenderOptions) -> Result<String> {
    let graph = Graph::from_state(state);
    let filter = match &options.pattern {
        Some(pattern) => Some(graph.build_filter(pattern)?),
        None => None,
    };
    let has_filter = filter.is_some();
    let mut walk = Walk {
        graph: &graph,
        collapse_instances: options.collapse_instances,
        // The collapse flags act only with a pattern.
        collapse_ancestors: options.collapse_ancestors && has_filter,
        collapse_descendants: options.collapse_descendants && has_filter,
        filter,
        expanded: HashSet::new(),
        output: String::new(),
    };
    walk.run();
    Ok(walk.output)
}

/// One node's display data.
struct NodeInfo<'a> {
    /// Display name.
    name: &'a str,
    /// Child node ids, in reference order.
    child_nodes: Vec<u32>,
    /// Child object ids, in reference order.
    child_objects: Vec<u32>,
}

/// The scene graph flattened to `u32` ids, so rendering never names a branded
/// id. Placement counts drive the instanced and unplaced marks: a node's count
/// is its parent references plus a root listing, an object's its parent
/// references.
struct Graph<'a> {
    /// Node display data by id.
    nodes: HashMap<u32, NodeInfo<'a>>,
    /// Node ids in listing order, for the unplaced section.
    node_order: Vec<u32>,
    /// Placement count per node id.
    node_placements: HashMap<u32, usize>,
    /// Object names by id.
    object_names: HashMap<u32, &'a str>,
    /// Object ids in listing order, for the unplaced section.
    object_order: Vec<u32>,
    /// Placement count per object id.
    object_placements: HashMap<u32, usize>,
    /// Root node ids, in listing order.
    roots: Vec<u32>,
}

impl<'a> Graph<'a> {
    /// Flattens `state` into the id-keyed graph, tallying placements as it goes.
    fn from_state(state: &'a VoxMain) -> Graph<'a> {
        let roots: Vec<u32> = state
            .root_hierarchy_nodes()
            .iter()
            .map(|root| root.to_u32())
            .collect();

        let mut nodes = HashMap::new();
        let mut node_order = Vec::new();
        let mut node_placements: HashMap<u32, usize> = HashMap::new();
        let mut object_placements: HashMap<u32, usize> = HashMap::new();
        for &root in &roots {
            *node_placements.entry(root).or_insert(0) += 1;
        }
        for (id, node) in state.iter_hierarchy_nodes() {
            let id = id.to_u32();
            node_order.push(id);
            let child_nodes: Vec<u32> = node.child_nodes.iter().map(|c| c.to_u32()).collect();
            let child_objects: Vec<u32> = node.child_objects.iter().map(|c| c.to_u32()).collect();
            for &child in &child_nodes {
                *node_placements.entry(child).or_insert(0) += 1;
            }
            for &object in &child_objects {
                *object_placements.entry(object).or_insert(0) += 1;
            }
            nodes.insert(
                id,
                NodeInfo {
                    name: node.name.as_str(),
                    child_nodes,
                    child_objects,
                },
            );
        }

        let mut object_names = HashMap::new();
        let mut object_order = Vec::new();
        for (id, object) in state.iter_objects() {
            let id = id.to_u32();
            object_order.push(id);
            object_names.insert(id, object.name());
        }

        Graph {
            nodes,
            node_order,
            node_placements,
            object_names,
            object_order,
            object_placements,
            roots,
        }
    }

    /// Placement count of node `id`.
    fn node_placement(&self, id: u32) -> usize {
        self.node_placements.get(&id).copied().unwrap_or(0)
    }

    /// Placement count of object `id`.
    fn object_placement(&self, id: u32) -> usize {
        self.object_placements.get(&id).copied().unwrap_or(0)
    }

    /// Display name of node `id`, or `""` when it does not resolve.
    fn node_name(&self, id: u32) -> &str {
        self.nodes.get(&id).map(|node| node.name).unwrap_or("")
    }

    /// Node ids that are neither a root nor a child, in listing order.
    fn unplaced_nodes(&self) -> Vec<u32> {
        self.node_order
            .iter()
            .copied()
            .filter(|&id| self.node_placement(id) == 0)
            .collect()
    }

    /// Object ids that no node places, in listing order.
    fn orphan_objects(&self) -> Vec<u32> {
        self.object_order
            .iter()
            .copied()
            .filter(|&id| self.object_placement(id) == 0)
            .collect()
    }

    /// Every node path, as `(path, node id)`: the roots' subtrees then the
    /// unplaced nodes' subtrees, each path the chain of node names from its
    /// section root. A node reached through several parents yields one path per
    /// placement. A node on its own ancestor chain is recorded once and not
    /// re-entered, so a cyclic document still terminates.
    fn enumerate_node_paths(&self) -> Vec<(String, u32)> {
        let mut out = Vec::new();
        let mut branch = HashSet::new();
        for &root in &self.roots {
            let path = self.node_name(root).to_string();
            self.enumerate_from(root, path, &mut branch, &mut out);
        }
        for id in self.unplaced_nodes() {
            let path = self.node_name(id).to_string();
            self.enumerate_from(id, path, &mut branch, &mut out);
        }
        out
    }

    /// Records `path` for node `id`, then descends into its child nodes unless
    /// `id` is already on the current branch (a cycle). `branch` is the set of
    /// node ids on the path from the section root to here.
    fn enumerate_from(
        &self,
        id: u32,
        path: String,
        branch: &mut HashSet<u32>,
        out: &mut Vec<(String, u32)>,
    ) {
        let Some(info) = self.nodes.get(&id) else {
            return;
        };
        out.push((path.clone(), id));
        if !branch.insert(id) {
            return;
        }
        for &child in &info.child_nodes {
            let child_path = child_path(&path, self.node_name(child));
            self.enumerate_from(child, child_path, branch, out);
        }
        branch.remove(&id);
    }

    /// Builds the path filter for `pattern`: the node paths it matches and every
    /// proper prefix of a match. The pattern is normalized with a leading `**/`
    /// through [`PathGlob`]. Errors on a malformed glob or when nothing matches.
    fn build_filter(&self, pattern: &str) -> Result<Filter> {
        let glob: PathGlob = pattern
            .parse()
            .map_err(|message| IOError::new(ErrorKind::InvalidInput, message))?;
        let paths = self.enumerate_node_paths();
        let candidates: Vec<&str> = paths.iter().map(|(path, _)| path.as_str()).collect();
        let flags = implementation::match_glob(glob.pattern(), &candidates)?;

        let matches: Vec<(String, u32)> = paths
            .iter()
            .zip(&flags)
            .filter(|&(_, &matched)| matched)
            .map(|((path, id), _)| (path.clone(), *id))
            .collect();
        if matches.is_empty() {
            return Err(IOError::new(
                ErrorKind::NotFound,
                format!("no node matched pattern '{pattern}'"),
            )
            .into());
        }

        let matched: HashSet<String> = matches.iter().map(|(path, _)| path.clone()).collect();
        let mut ancestors = HashSet::new();
        for (path, _) in &matches {
            let parts: Vec<&str> = path.split('/').collect();
            for end in 1..parts.len() {
                ancestors.insert(parts[..end].join("/"));
            }
        }
        Ok(Filter {
            matches,
            matched,
            ancestors,
        })
    }
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

/// A `pattern`'s resolved matches. `matches` is every matched node path with its
/// node id, in enumeration order; `matched` and `ancestors` are the same paths
/// and their proper prefixes, for `O(1)` lookups while walking.
struct Filter {
    /// Matched node paths with their node ids, in enumeration order.
    matches: Vec<(String, u32)>,
    /// The matched node paths.
    matched: HashSet<String>,
    /// Every proper prefix of a matched path.
    ancestors: HashSet<String>,
}

impl Filter {
    /// True if `path` is a match or leads to one, so its subtree stays visible.
    fn on_path(&self, path: &str) -> bool {
        self.matched.contains(path) || self.ancestors.contains(path)
    }
}

/// One render pass over a [`Graph`]: the immutable options and filter, plus the
/// growing output and the `expanded` memory that collapses repeat instances.
struct Walk<'a> {
    /// The graph being rendered.
    graph: &'a Graph<'a>,
    /// Collapse repeat instances to a stub after the first placement.
    collapse_instances: bool,
    /// Hide each match's ancestor chain behind an `[Ancestors]` marker.
    collapse_ancestors: bool,
    /// Hide each match's descendants behind a `[Descendants]` marker.
    collapse_descendants: bool,
    /// The path filter, when a `pattern` was given.
    filter: Option<Filter>,
    /// Node ids whose first placement has already expanded.
    expanded: HashSet<u32>,
    /// The accumulated tree text.
    output: String,
}

impl Walk<'_> {
    /// Renders the whole graph into `output`.
    fn run(&mut self) {
        if self.collapse_ancestors {
            self.run_collapsed_ancestors();
        } else {
            self.run_sections();
        }
    }

    /// The `Root` section of each root's subtree, then the `Unplaced` section of
    /// nodes that are neither a root nor a child and objects no node places. With
    /// a filter, each section keeps only entries on the way to a match, and a
    /// section header prints only when its section has visible entries.
    fn run_sections(&mut self) {
        let roots = self.graph.roots.clone();
        self.render_group("Root", &roots, &[], false);
        let unplaced = self.graph.unplaced_nodes();
        let orphans = self.graph.orphan_objects();
        self.render_group("Unplaced", &unplaced, &orphans, true);
    }

    /// Prints matches as a flat list, each match's subtree behind an
    /// `[Ancestors]` marker, dropped when the match is a section root. Runs only
    /// with a filter, so the ancestor chain being hidden is well defined.
    fn run_collapsed_ancestors(&mut self) {
        let matches = match &self.filter {
            Some(filter) => filter.matches.clone(),
            None => return,
        };
        let total = matches.len();
        for (index, (path, id)) in matches.iter().enumerate() {
            let is_last = index + 1 == total;
            let mut branch = HashSet::new();
            if path.contains('/') {
                let connector = if is_last {
                    CONNECTOR_LAST
                } else {
                    CONNECTOR_MID
                };
                self.output.push_str(&format!("{connector} [Ancestors]\n"));
                let extension = if is_last {
                    EXTENSION_LAST
                } else {
                    EXTENSION_MID
                };
                self.render_node(*id, extension, true, path, true, &mut branch);
            } else {
                self.render_node(*id, "", is_last, path, true, &mut branch);
            }
        }
    }

    /// Renders one section under `header`: its top-level node ids then object
    /// ids, at prefix depth zero. With a filter, a top-level node shows only when
    /// it is on the way to a match, and orphan objects, which no node path names,
    /// are dropped. Nothing prints, header included, when the section is empty.
    fn render_group(&mut self, header: &str, node_ids: &[u32], object_ids: &[u32], gap: bool) {
        let nodes: Vec<u32> = node_ids
            .iter()
            .copied()
            .filter(|&id| self.top_visible(id))
            .collect();
        let objects: Vec<u32> = if self.filter.is_some() {
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

        let mut branch = HashSet::new();
        for (index, &id) in nodes.iter().enumerate() {
            let path = self.graph.node_name(id).to_string();
            self.render_node(id, "", index + 1 == total, &path, false, &mut branch);
        }
        for (index, &id) in objects.iter().enumerate() {
            let last = nodes.len() + index + 1 == total;
            self.render_object(id, "", last);
        }
    }

    /// Whether a section-root node shows: always without a filter, else only when
    /// its own path is on the way to a match.
    fn top_visible(&self, id: u32) -> bool {
        match &self.filter {
            None => true,
            Some(filter) => filter.on_path(self.graph.node_name(id)),
        }
    }

    /// Appends node `id`'s subtree at `path`. Every line ends with a
    /// `[Node <id>, ...]` tag: an instanced node adds `Instance` at every
    /// placement, and with `collapse_instances` only the first placement expands
    /// while later ones add `Collapsed` and stop. A node on its own ancestor
    /// chain adds `Cycle` and stops, so a document that skipped validation cannot
    /// recurse forever. `in_match` is set once this or an ancestor matched: below
    /// a match the whole subtree shows, but `collapse_descendants` replaces it
    /// with a `[Descendants]` marker; above a match a filter keeps only the child
    /// nodes leading to one.
    fn render_node(
        &mut self,
        id: u32,
        prefix: &str,
        is_last: bool,
        path: &str,
        in_match: bool,
        branch: &mut HashSet<u32>,
    ) {
        let graph = self.graph;
        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };
        let Some(info) = graph.nodes.get(&id) else {
            self.output
                .push_str(&format!("{prefix}{connector} [missing node {id}]\n"));
            return;
        };

        let instanced = graph.node_placement(id) >= 2;
        let is_cycle = branch.contains(&id);
        // Only a repeat placement collapses, and only outside a cycle; the
        // `insert` both records the first placement and reports later ones.
        let collapsed_stub =
            self.collapse_instances && instanced && !is_cycle && !self.expanded.insert(id);
        let matched = self
            .filter
            .as_ref()
            .is_some_and(|f| f.matched.contains(path));
        let in_match = in_match || matched;

        let mut tag = format!("Node {id}");
        if instanced {
            tag.push_str(", Instance");
        }
        if is_cycle {
            tag.push_str(", Cycle");
        } else if collapsed_stub {
            tag.push_str(", Collapsed");
        }
        self.output
            .push_str(&format!("{prefix}{connector} {} [{tag}]\n", info.name));
        if is_cycle || collapsed_stub {
            return;
        }

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };
        let child_prefix = format!("{prefix}{extension}");

        // In a match, collapse the whole subtree into one marker.
        let has_children = !info.child_nodes.is_empty() || !info.child_objects.is_empty();
        if self.collapse_descendants && in_match && has_children {
            self.output
                .push_str(&format!("{child_prefix}{CONNECTOR_LAST} [Descendants]\n"));
            return;
        }

        // In a match, or with no filter, show every child; above a match keep
        // only the child nodes leading to one, dropping objects, which no node
        // path names.
        let show_all = self.filter.is_none() || in_match;
        let mut child_nodes: Vec<(u32, String)> = Vec::new();
        for &child in &info.child_nodes {
            let child_path = child_path(path, graph.node_name(child));
            if show_all || self.filter.as_ref().is_some_and(|f| f.on_path(&child_path)) {
                child_nodes.push((child, child_path));
            }
        }
        let child_objects: &[u32] = if show_all { &info.child_objects } else { &[] };
        let child_count = child_nodes.len() + child_objects.len();

        branch.insert(id);
        for (index, (child, child_path)) in child_nodes.iter().enumerate() {
            let last = index + 1 == child_count;
            self.render_node(*child, &child_prefix, last, child_path, in_match, branch);
        }
        for (index, &object) in child_objects.iter().enumerate() {
            let last = child_nodes.len() + index + 1 == child_count;
            self.render_object(object, &child_prefix, last);
        }
        branch.remove(&id);
    }

    /// Appends object `id` as a leaf line ending with an `[Object <id>, ...]`
    /// tag: it adds `Instance` when the object is placed by two or more nodes.
    fn render_object(&mut self, id: u32, prefix: &str, is_last: bool) {
        let graph = self.graph;
        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };
        let Some(&name) = graph.object_names.get(&id) else {
            self.output
                .push_str(&format!("{prefix}{connector} [missing object {id}]\n"));
            return;
        };
        let mut tag = format!("Object {id}");
        if graph.object_placement(id) >= 2 {
            tag.push_str(", Instance");
        }
        self.output
            .push_str(&format!("{prefix}{connector} {name} [{tag}]\n"));
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Result,
        implementation::hierarchy_show::{RenderOptions, render},
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
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
        render(
            state,
            &RenderOptions {
                pattern: pattern.map(str::to_owned),
                collapse_instances,
                collapse_ancestors,
                collapse_descendants,
            },
        )
    }

    /// A 1x1x1 object; only its name matters to the tree.
    fn object(name: &str) -> VoxObject {
        VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap()
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
            "Root\n\
             \u{2514} root [Node 0]\n\
             \u{20}\u{20}\u{2514} body [Object 0]\n"
        );
    }

    #[test]
    fn markdown_marks_every_instance_by_default() {
        let output = show(&instanced_state(), None, false, false, false);
        assert_eq!(
            output,
            "Root\n\
             \u{2514} root [Node 3]\n\
             \u{20}\u{20}\u{251C} armA [Node 1]\n\
             \u{20}\u{20}\u{2502} \u{2514} leaf [Node 0, Instance]\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} head [Object 0]\n\
             \u{20}\u{20}\u{2514} armB [Node 2]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} leaf [Node 0, Instance]\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} head [Object 0]\n"
        );
    }

    #[test]
    fn collapse_instances_stubs_repeat_placements() {
        let output = show(&instanced_state(), None, true, false, false);
        assert_eq!(
            output,
            "Root\n\
             \u{2514} root [Node 3]\n\
             \u{20}\u{20}\u{251C} armA [Node 1]\n\
             \u{20}\u{20}\u{2502} \u{2514} leaf [Node 0, Instance]\n\
             \u{20}\u{20}\u{2502} \u{20}\u{20}\u{2514} head [Object 0]\n\
             \u{20}\u{20}\u{2514} armB [Node 2]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} leaf [Node 0, Instance, Collapsed]\n"
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
            "Root\n\
             \u{2514} root [Node 0]\n\
             \u{20}\u{20}\u{2514} body [Object 0]\n\
             \n\
             Unplaced\n\
             \u{251C} spareNode [Node 1]\n\
             \u{2502} \u{2514} spareChild [Object 2]\n\
             \u{2514} looseMesh [Object 1]\n"
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
        assert!(output.contains("Cycle"), "output was:\n{output}");
        assert_eq!(output.matches("[Node ").count(), 3);
    }

    #[test]
    fn pattern_keeps_only_matches_and_their_ancestors() {
        // `**/hand` matches `root/armA/hand`; the `armB`/`foot` branch is pruned,
        // and the matched node's object shows.
        let output = show(&pattern_state(), Some("hand"), false, false, false);
        assert_eq!(
            output,
            "Root\n\
             \u{2514} root [Node 4]\n\
             \u{20}\u{20}\u{2514} armA [Node 2]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} hand [Node 0]\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} handMesh [Object 0]\n"
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
            "\u{2514} [Ancestors]\n\
             \u{20}\u{20}\u{2514} hand [Node 0]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} handMesh [Object 0]\n"
        );
    }

    #[test]
    fn collapse_descendants_hides_the_subtree_below_a_match() {
        let output = show(&pattern_state(), Some("hand"), false, false, true);
        assert_eq!(
            output,
            "Root\n\
             \u{2514} root [Node 4]\n\
             \u{20}\u{20}\u{2514} armA [Node 2]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} hand [Node 0]\n\
             \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{2514} [Descendants]\n"
        );
    }

    #[test]
    fn collapse_ancestors_and_descendants_combine() {
        let output = show(&pattern_state(), Some("hand"), false, true, true);
        assert_eq!(
            output,
            "\u{2514} [Ancestors]\n\
             \u{20}\u{20}\u{2514} hand [Node 0]\n\
             \u{20}\u{20}\u{20}\u{20}\u{2514} [Descendants]\n"
        );
    }

    #[test]
    fn collapse_flags_do_nothing_without_a_pattern() {
        let plain = show(&pattern_state(), None, false, false, false);
        let with_flags = show(&pattern_state(), None, false, true, true);
        assert_eq!(plain, with_flags);
    }
}
