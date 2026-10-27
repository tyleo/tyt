use crate::{Error, Result};
use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxExt, VoxMain};

/// A hierarchy-node id.
type NodeId = U32Id<BVoxHierarchyNode>;

/// An object id.
type ObjectId = U32Id<BVoxObject>;

/// Scales the scene about the world origin by `scale`, multiplying each root
/// node's position and scale. Errors, changing nothing, when:
///
/// 1. `scale` is not positive and finite
/// 2. `scale` overflows a root's position or scale
/// 3. a root is also a child, which would scale it twice
/// 4. an object sits under no root
pub fn scale_scene<T: VoxExt>(main: &mut VoxMain<T>, scale: f64) -> Result<()> {
    if !(scale.is_finite() && scale > 0.0) {
        return Err(Error::invalid(format!(
            "scene scale {scale} is not positive and finite"
        )));
    }

    let root_ids = main.root_hierarchy_node_ids().to_vec();

    let roots: HashSet<NodeId> = root_ids.iter().copied().collect();

    for (node_id, node) in main.iter_hierarchy_nodes() {
        if let Some(child_id) = node
            .child_node_ids
            .iter()
            .find(|child_id| roots.contains(child_id))
        {
            return Err(Error::invalid(format!(
                "root node {child_id} is also a child of node {node_id}, so it would take the scene scale twice"
            )));
        }
    }

    let placed = placed_objects(main, &root_ids);

    if let Some((object_id, object)) = main
        .iter_objects()
        .find(|(object_id, _)| !placed.contains(object_id))
    {
        return Err(Error::invalid(format!(
            "no root node places object {object_id} (\"{}\"), so the scene scale cannot reach it",
            object.name()
        )));
    }

    let mut scaled = Vec::with_capacity(root_ids.len());

    for root_id in root_ids {
        let mut transform = main
            .hierarchy_node(root_id)
            .expect("a root is one of the main's nodes")
            .transform;

        transform.position *= scale;

        transform.scale *= scale;

        if !(transform.position.is_finite() && transform.scale.is_finite()) {
            return Err(Error::invalid(format!(
                "scene scale {scale} takes root node {root_id} past the finite range"
            )));
        }

        scaled.push((root_id, transform));
    }

    for (root_id, transform) in scaled {
        main.set_hierarchy_node_transform(root_id, transform)?;
    }

    Ok(())
}

/// The objects the subtrees of `root_ids` place.
fn placed_objects<T: VoxExt>(main: &VoxMain<T>, root_ids: &[NodeId]) -> HashSet<ObjectId> {
    let mut visited = HashSet::new();

    let mut placed = HashSet::new();

    let mut stack = root_ids.to_vec();

    while let Some(node_id) = stack.pop() {
        if !visited.insert(node_id) {
            continue;
        }

        let node = main
            .hierarchy_node(node_id)
            .expect("a walked node is one of the main's");

        placed.extend(node.child_object_ids.iter().copied());

        stack.extend(node.child_node_ids.iter().copied());
    }

    placed
}

#[cfg(test)]
mod tests {
    use crate::{operations::vox_doc::scale_scene, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject};

    /// A root `root` at `(1, 2, 3)` with scale 0.5 placing child `child` at
    /// `(4, 0, 0)`, which places object `a`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let object_id = main
            .retain_object(VoxObject::new("a".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap())
            .unwrap();

        let child_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "child".to_owned(),
                transform: TyTransformF64 {
                    position: TyVector3F64::new(4.0, 0.0, 0.0),
                    ..Default::default()
                },
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();

        let root_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "root".to_owned(),
                transform: TyTransformF64 {
                    position: TyVector3F64::new(1.0, 2.0, 3.0),
                    scale: TyVector3F64::splat(0.5),
                    ..Default::default()
                },
                child_node_ids: vec![child_id],
                ..Default::default()
            })
            .unwrap();

        main.set_root_hierarchy_node_ids(vec![root_id]).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn scales_each_root_position_and_scale() {
        let mut main = scene();

        scale_scene(&mut main, 100.0).unwrap();

        let root = main.hierarchy_node(U32Id::from_u32(1)).unwrap();

        assert_eq!(
            root.transform.position,
            TyVector3F64::new(100.0, 200.0, 300.0)
        );
        assert_eq!(root.transform.scale, TyVector3F64::splat(50.0));

        let child = main.hierarchy_node(U32Id::from_u32(0)).unwrap();

        assert_eq!(child.transform.position, TyVector3F64::new(4.0, 0.0, 0.0));
        assert_eq!(child.transform.scale, TyVector3F64::ONE);
        assert_eq!(HookRecorder::events(&main), ["node 1 transform set"]);
        main.validate().unwrap();
    }

    #[test]
    fn a_scale_not_positive_and_finite_is_an_error_that_changes_nothing() {
        let mut main = scene();

        for scale in [0.0, -1.0, f64::INFINITY, f64::NAN] {
            assert!(scale_scene(&mut main, scale).is_err());
        }

        assert!(HookRecorder::events(&main).is_empty());
    }

    #[test]
    fn a_scale_overflowing_a_root_is_an_error_that_changes_nothing() {
        let mut main = scene();

        let error = scale_scene(&mut main, f64::MAX).unwrap_err();

        assert!(error.to_string().contains("finite range"), "{error}");
        assert!(HookRecorder::events(&main).is_empty());
    }

    #[test]
    fn a_root_that_is_also_a_child_is_an_error_that_changes_nothing() {
        let mut main = scene();

        main.set_root_hierarchy_node_ids(vec![U32Id::from_u32(1), U32Id::from_u32(0)])
            .unwrap();

        let error = scale_scene(&mut main, 2.0).unwrap_err();

        assert!(error.to_string().contains("twice"), "{error}");
        assert_eq!(HookRecorder::events(&main), ["roots set"]);
    }

    #[test]
    fn an_object_no_root_places_is_an_error_that_changes_nothing() {
        let mut main = scene();

        main.set_root_hierarchy_node_ids(Vec::new()).unwrap();

        let error = scale_scene(&mut main, 2.0).unwrap_err();

        assert!(error.to_string().contains("cannot reach"), "{error}");
        assert_eq!(HookRecorder::events(&main), ["roots set"]);
    }
}
