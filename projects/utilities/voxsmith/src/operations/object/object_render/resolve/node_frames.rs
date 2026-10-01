use crate::{
    Error, Result,
    operations::object::RenderElement,
    utilities::{NodePath, is_node_path_match, node_paths},
};
use pathspec::GitIgnoreRegex;
use ty_math::TyTransformF64;
use voxcore::{VoxExt, VoxMain};

/// The frame each hierarchy node path offers a `node` transform: the path's
/// world transform with the voxel size applied, the one the flatten gives the
/// path's placements.
pub struct NodeFrames {
    frames: Vec<(String, TyTransformF64)>,
}

impl NodeFrames {
    /// The frames of every node path of `main` at `voxel_size` meters per
    /// voxel.
    pub fn new<T: VoxExt>(main: &VoxMain<T>, voxel_size: f64) -> Self {
        NodeFrames {
            frames: node_paths(main)
                .into_iter()
                .map(|NodePath { path, world, .. }| {
                    (
                        path,
                        TyTransformF64 {
                            position: world.position * voxel_size,
                            ..world
                        },
                    )
                })
                .collect(),
        }
    }

    /// The frame of the one node path the glob `path` matches. Errors,
    /// reporting `element`, if the glob does not parse or matches no path or
    /// several.
    pub fn frame(&self, element: &RenderElement, path: &str) -> Result<TyTransformF64> {
        let patterns = GitIgnoreRegex::from_spans_ignore_inert(&[path])?;

        let matched: Vec<&(String, TyTransformF64)> = self
            .frames
            .iter()
            .filter(|(candidate, _)| is_node_path_match(&patterns, candidate))
            .collect();

        match matched.as_slice() {
            [(_, frame)] => Ok(*frame),

            [] => Err(Error::render_record(
                element.clone(),
                format!("reads the node path glob `{path}`, which matches no node path"),
            )),

            several => {
                let listed: Vec<&str> = several.iter().map(|(path, _)| path.as_str()).collect();

                Err(Error::render_record(
                    element.clone(),
                    format!(
                        "reads the node path glob `{path}`, which matches {} node paths: {}",
                        several.len(),
                        listed.join(", ")
                    ),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, Result,
        operations::object::{NodeFrames, RenderElement},
    };
    use branded_id::U32Id;
    use ty_math::{TyTransformF64, TyVector3F64};
    use voxcore::{BVoxHierarchyNode, VoxHierarchyNode, VoxMain};

    fn element() -> RenderElement {
        RenderElement::ViewTransform {
            name: "ride".to_owned(),
        }
    }

    /// `house` holds `door`, and `garage` holds the same `door`. `house` sits
    /// at `x = 10`. `door` sits one unit above its parent.
    fn scene() -> VoxMain {
        let mut main = VoxMain::default();

        let node = |main: &mut VoxMain,
                    name: &str,
                    position: TyVector3F64,
                    child_node_ids: Vec<U32Id<BVoxHierarchyNode>>| {
            main.retain_hierarchy_node(VoxHierarchyNode {
                name: name.to_owned(),
                transform: TyTransformF64::from_translation(position),
                child_node_ids,
                ..Default::default()
            })
            .unwrap()
        };

        let door_id = node(&mut main, "door", TyVector3F64::Y, vec![]);
        let house_id = node(&mut main, "house", TyVector3F64::X * 10.0, vec![door_id]);
        let garage_id = node(&mut main, "garage", TyVector3F64::ZERO, vec![door_id]);
        main.push_root_hierarchy_node_id(house_id).unwrap();
        main.push_root_hierarchy_node_id(garage_id).unwrap();

        main
    }

    fn reason(result: Result<TyTransformF64>) -> String {
        match result {
            Err(Error::RenderRecord { element, reason }) => {
                assert_eq!(element, self::element());
                reason
            }

            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_glob_matching_one_path_gives_its_frame_at_the_voxel_size() {
        let frames = NodeFrames::new(&scene(), 0.5);

        let door = frames.frame(&element(), "house/door").unwrap();
        assert_eq!(door.position, TyVector3F64::new(5.0, 0.5, 0.0));
        assert_eq!(door.scale, TyVector3F64::ONE);

        assert_eq!(
            frames.frame(&element(), "garage").unwrap().position,
            TyVector3F64::ZERO
        );
    }

    #[test]
    fn a_glob_matching_no_path_or_several_errors_listing_them() {
        let frames = NodeFrames::new(&scene(), 1.0);

        assert_eq!(
            reason(frames.frame(&element(), "shed")),
            "reads the node path glob `shed`, which matches no node path"
        );

        assert_eq!(
            reason(frames.frame(&element(), "door")),
            "reads the node path glob `door`, which matches 2 node paths: house/door, garage/door"
        );
    }
}
