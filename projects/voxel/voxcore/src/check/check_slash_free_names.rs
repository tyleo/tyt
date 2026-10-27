use crate::{VoxState, check::VoxCheck};

/// Checks that no hierarchy node or object name holds `/`, which a joined
/// hierarchy path reads as a step to a child.
pub fn check_slash_free_names(state: &VoxState) -> VoxCheck {
    let nodes = state
        .iter_hierarchy_nodes()
        .filter(|(_, node)| node.name.contains('/'))
        .map(|(node_id, node)| {
            format!("hierarchy node {node_id} name {:?} holds \"/\"", node.name)
        });

    let objects = state
        .iter_objects()
        .filter(|(_, object)| object.name().contains('/'))
        .map(|(object_id, object)| {
            format!("object {object_id} name {:?} holds \"/\"", object.name())
        });

    let messages: Vec<String> = nodes.chain(objects).collect();

    if messages.is_empty() {
        VoxCheck::passed("slash-free-names")
    } else {
        VoxCheck::failed("slash-free-names", messages)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        VoxHierarchyNode, VoxMain, VoxObject,
        check::{VoxCheck, check_slash_free_names},
    };
    use ty_math::TyVector3U32;

    fn main_named(node_names: &[&str], object_names: &[&str]) -> VoxMain {
        let mut main = VoxMain::default();

        for name in node_names {
            let node = VoxHierarchyNode {
                name: name.to_string(),
                ..Default::default()
            };

            main.retain_hierarchy_node(node).unwrap();
        }

        for name in object_names {
            let object = VoxObject::new(name.to_string(), TyVector3U32::new(1, 1, 1)).unwrap();

            main.retain_object(object).unwrap();
        }

        main
    }

    #[test]
    fn names_without_a_slash_pass() {
        let main = main_named(&["house", "door"], &["knob"]);

        assert_eq!(
            check_slash_free_names(main.state()),
            VoxCheck::passed("slash-free-names")
        );
    }

    #[test]
    fn each_slashed_name_fails_with_its_id_nodes_first() {
        let main = main_named(&["house", "a/b"], &["door/knob", "lid"]);

        assert_eq!(
            check_slash_free_names(main.state()),
            VoxCheck::failed(
                "slash-free-names",
                vec![
                    "hierarchy node 1 name \"a/b\" holds \"/\"".to_owned(),
                    "object 0 name \"door/knob\" holds \"/\"".to_owned(),
                ]
            )
        );
    }
}
