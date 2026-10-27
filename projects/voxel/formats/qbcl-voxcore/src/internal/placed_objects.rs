use voxcore::{VoxExt, VoxHierarchyNode, VoxMain, VoxObject};

/// The objects a hierarchy node places, in order.
pub fn placed_objects<'a, T: VoxExt>(
    hierarchy: &VoxHierarchyNode,
    main: &'a VoxMain<T>,
) -> Vec<&'a VoxObject> {
    hierarchy
        .child_object_ids
        .iter()
        .map(|&object_id| {
            main.object(object_id)
                .expect("a placed object is one of the state's")
        })
        .collect()
}
