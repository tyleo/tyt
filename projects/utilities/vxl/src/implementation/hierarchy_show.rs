use crate::{Format, Result, implementation};
use std::{
    collections::{HashMap, HashSet},
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
pub fn hierarchy_show(input: &Path, from: Option<Format>, collapse_instances: bool) -> Result<()> {
    let state = implementation::load_state(input, from)?;
    let output = render(&state, collapse_instances);
    implementation::write_stdout(output.as_bytes())
}

/// Renders the scene graph of `state` as a tree, the testable core of
/// [`hierarchy_show`].
fn render(state: &VoxMain, collapse_instances: bool) -> String {
    Graph::from_state(state).render_markdown(collapse_instances)
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

    /// The tree: a `Root` section of each root's subtree, then an `Unplaced`
    /// section listing nodes that are neither a root nor a child and objects no
    /// node places. A section header prints only when its section is non-empty.
    fn render_markdown(&self, collapse_instances: bool) -> String {
        let mut output = String::new();
        let mut expanded = HashSet::new();
        let mut ancestors = HashSet::new();
        if !self.roots.is_empty() {
            output.push_str("Root\n");
            let root_count = self.roots.len();
            for (index, &root) in self.roots.iter().enumerate() {
                self.render_node(
                    root,
                    "",
                    index + 1 == root_count,
                    collapse_instances,
                    &mut expanded,
                    &mut ancestors,
                    &mut output,
                );
            }
        }

        let unplaced_nodes: Vec<u32> = self
            .node_order
            .iter()
            .copied()
            .filter(|&id| self.node_placement(id) == 0)
            .collect();
        let orphan_objects: Vec<u32> = self
            .object_order
            .iter()
            .copied()
            .filter(|&id| self.object_placement(id) == 0)
            .collect();
        let unplaced_count = unplaced_nodes.len() + orphan_objects.len();
        if unplaced_count > 0 {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str("Unplaced\n");
            for (index, &id) in unplaced_nodes.iter().enumerate() {
                self.render_node(
                    id,
                    "",
                    index + 1 == unplaced_count,
                    collapse_instances,
                    &mut expanded,
                    &mut ancestors,
                    &mut output,
                );
            }
            for (index, &id) in orphan_objects.iter().enumerate() {
                let last = unplaced_nodes.len() + index + 1 == unplaced_count;
                self.render_object(id, "", last, &mut output);
            }
        }
        output
    }

    /// Appends node `id`'s subtree. Every line ends with a `[Node <id>, ...]`
    /// tag: an instanced node (two or more placements) adds `Instance` at every
    /// placement, and with `collapse_instances` only its first placement expands
    /// while later ones add `Collapsed` and stop. A node found on its own
    /// ancestor chain adds `Cycle` and is not re-entered, so a document that
    /// skipped validation cannot recurse forever.
    #[allow(clippy::too_many_arguments)]
    fn render_node(
        &self,
        id: u32,
        prefix: &str,
        is_last: bool,
        collapse_instances: bool,
        expanded: &mut HashSet<u32>,
        ancestors: &mut HashSet<u32>,
        output: &mut String,
    ) {
        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };
        let Some(info) = self.nodes.get(&id) else {
            output.push_str(&format!("{prefix}{connector} [missing node {id}]\n"));
            return;
        };

        let instanced = self.node_placement(id) >= 2;
        let is_cycle = ancestors.contains(&id);
        // Only a repeat placement collapses, and only outside a cycle; the
        // `insert` both records the first placement and reports later ones.
        let collapsed_stub = collapse_instances && instanced && !is_cycle && !expanded.insert(id);
        let mut tag = format!("Node {id}");
        if instanced {
            tag.push_str(", Instance");
        }
        if is_cycle {
            tag.push_str(", Cycle");
        } else if collapsed_stub {
            tag.push_str(", Collapsed");
        }
        output.push_str(&format!("{prefix}{connector} {} [{tag}]\n", info.name));
        if is_cycle || collapsed_stub {
            return;
        }

        let extension = if is_last {
            EXTENSION_LAST
        } else {
            EXTENSION_MID
        };
        let child_prefix = format!("{prefix}{extension}");
        let child_count = info.child_nodes.len() + info.child_objects.len();
        ancestors.insert(id);
        for (index, &child) in info.child_nodes.iter().enumerate() {
            self.render_node(
                child,
                &child_prefix,
                index + 1 == child_count,
                collapse_instances,
                expanded,
                ancestors,
                output,
            );
        }
        for (index, &object) in info.child_objects.iter().enumerate() {
            let last = info.child_nodes.len() + index + 1 == child_count;
            self.render_object(object, &child_prefix, last, output);
        }
        ancestors.remove(&id);
    }

    /// Appends object `id` as a leaf line ending with an `[Object <id>, ...]`
    /// tag: it adds `Instance` when the object is placed by two or more nodes.
    fn render_object(&self, id: u32, prefix: &str, is_last: bool, output: &mut String) {
        let connector = if is_last {
            CONNECTOR_LAST
        } else {
            CONNECTOR_MID
        };
        let Some(&name) = self.object_names.get(&id) else {
            output.push_str(&format!("{prefix}{connector} [missing object {id}]\n"));
            return;
        };
        let mut tag = format!("Object {id}");
        if self.object_placement(id) >= 2 {
            tag.push_str(", Instance");
        }
        output.push_str(&format!("{prefix}{connector} {name} [{tag}]\n"));
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::hierarchy_show::render;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};

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

    #[test]
    fn markdown_renders_a_simple_tree() {
        let output = render(&simple_state(), false);
        assert_eq!(
            output,
            "Root\n\
             \u{2514} root [Node 0]\n\
             \u{20}\u{20}\u{2514} body [Object 0]\n"
        );
    }

    #[test]
    fn markdown_marks_every_instance_by_default() {
        let output = render(&instanced_state(), false);
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
        let output = render(&instanced_state(), true);
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

        let output = render(&state, false);
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

        let output = render(&state, false);
        // `a` opens, `b` under it, then `a` again as a cycle leaf; it stops
        // there rather than recursing forever.
        assert!(output.contains("Cycle"), "output was:\n{output}");
        assert_eq!(output.matches("[Node ").count(), 3);
    }
}
