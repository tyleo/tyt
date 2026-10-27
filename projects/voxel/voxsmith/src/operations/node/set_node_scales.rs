use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::TyVector3F64;
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxMain};

/// Sets the transform scale of each node in `node_ids`. A negative component
/// mirrors that axis. Errors, changing nothing, when an id is not one of the
/// main's nodes or `scale` has a zero or non-finite component.
pub fn set_node_scales<T: VoxExt>(
    main: &mut VoxMain<T>,
    node_ids: &[U32Id<BVoxHierarchyNode>],
    scale: TyVector3F64,
) -> Result<()> {
    if !scale.is_finite() {
        return Err(Error::invalid(format!(
            "scale {scale} has a non-finite component"
        )));
    }

    if scale.cmpeq(TyVector3F64::ZERO).any() {
        return Err(Error::invalid(format!(
            "scale {scale} has a zero component"
        )));
    }

    for &node_id in node_ids {
        if main.hierarchy_node(node_id).is_none() {
            return Err(VoxError::UnknownHierarchyNode { node_id }.into());
        }
    }

    for &node_id in node_ids {
        let mut transform = main
            .hierarchy_node(node_id)
            .expect("node_ids are checked above")
            .transform;

        transform.scale = scale;

        main.set_hierarchy_node_transform(node_id, transform)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::set_node_scales, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyTransformF64, TyVector3F64};
    use voxcore::{VoxHierarchyNode, VoxMain};

    /// A node `a` at `(1, 2, 3)`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let a = VoxHierarchyNode {
            name: "a".to_owned(),
            transform: TyTransformF64 {
                position: TyVector3F64::new(1.0, 2.0, 3.0),
                ..Default::default()
            },
            ..Default::default()
        };

        main.retain_hierarchy_node(a).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn a_negative_component_mirrors_that_axis() {
        let mut main = scene();

        let scale = TyVector3F64::new(-1.0, 2.0, 0.5);

        set_node_scales(&mut main, &[U32Id::from_u32(0)], scale).unwrap();

        let a = main.hierarchy_node(U32Id::from_u32(0)).unwrap();

        assert_eq!(a.transform.scale, scale);
        assert_eq!(a.transform.position, TyVector3F64::new(1.0, 2.0, 3.0));
        assert_eq!(HookRecorder::events(&main), ["node 0 transform set"]);
        main.validate().unwrap();
    }

    #[test]
    fn a_zero_or_non_finite_component_is_an_error_that_changes_nothing() {
        let mut main = scene();

        for scale in [
            TyVector3F64::new(1.0, 0.0, 1.0),
            TyVector3F64::new(1.0, -0.0, 1.0),
            TyVector3F64::new(f64::INFINITY, 1.0, 1.0),
        ] {
            assert!(set_node_scales(&mut main, &[U32Id::from_u32(0)], scale).is_err());
        }

        assert!(HookRecorder::events(&main).is_empty());
    }
}
