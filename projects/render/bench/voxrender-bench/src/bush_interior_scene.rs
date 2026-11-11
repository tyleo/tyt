use crate::{eye_view, foliage_scene};
use ty_math::{TyVector3F64, TyVector3U32};
use voxrender::RenderScene;

/// The bounds of S4's bush.
const BUSH: TyVector3U32 = TyVector3U32::new(60, 46, 60);

/// S4's garden seen from inside its first bush, out through the leaves. The
/// eye sits inside a tile's box, and the boxes behind that box start where
/// it ends.
pub fn bush_interior_scene() -> RenderScene {
    let mut scene = foliage_scene();

    let view_ids: Vec<_> = scene.iter_views().map(|(view_id, _)| view_id).collect();

    for view_id in view_ids {
        scene
            .release_view(view_id)
            .expect("the view is the scene's");
    }

    let (bush, object) = scene
        .iter_placements()
        .filter_map(|(_, placement)| {
            let object = scene.object(placement.object_id)?;

            (object.bounds() == BUSH).then_some((placement.transform, object))
        })
        .next()
        .expect("the garden holds a bush");

    // The eye stands in an empty cell, where no material surrounds it.
    let cell = (24..32)
        .map(|x| TyVector3U32::new(x, 20, 24))
        .find(|&cell| {
            object
                .voxel_id(cell)
                .and_then(|voxel_id| object.voxel_material(voxel_id))
                .is_none()
        })
        .expect("the bush has an empty cell");

    let view = eye_view(
        bush.transform_point(cell.as_dvec3() + 0.5),
        bush.transform_point(TyVector3F64::new(60.0, 26.0, 52.0)),
    );

    scene.retain_view(view).expect("a bench view is valid");

    scene
}

#[cfg(test)]
mod tests {
    use crate::{bush_interior_scene, reference_hits};

    #[test]
    fn the_view_sees_leaves() {
        let scene = bush_interior_scene();
        let (view_id, _) = scene.iter_views().next().unwrap();
        let hits = reference_hits(&scene, view_id, 32).unwrap();

        assert!(hits.iter().filter(|hit| hit.is_some()).count() * 2 > hits.len());
    }
}
