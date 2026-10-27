use branded_id::U32Id;
use ty_math::{
    TyLinSrgbaF64, TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3F64,
    TyVector3U32,
};
use voxrender::{
    RenderPlacement, RenderProjection, RenderScene, RenderView,
    fixtures::{matte, solid_object},
};

/// An empty grid `depth` cells deep with a wall at its far end, seen from
/// inside the near end through a 90-degree view. Every pixel's ray walks
/// at least `depth - 1` steps to the wall, so the steps outweigh the
/// per-pixel setup.
pub fn far_wall_scene(depth: u32) -> RenderScene {
    let mut scene = RenderScene::default();

    let material_id = scene
        .retain_material(matte(TyLinSrgbaF64::new(0.8, 0.8, 0.8, 1.0)))
        .expect("an empty scene takes a material");

    // At 90 degrees the view spans `depth` either side of its axis at the
    // wall, so a grid twice as wide puts every ray on the wall.
    let bounds = TyVector3U32::new(2 * depth, 2 * depth, depth);
    let object_id = U32Id::from_u32(0);

    scene
        .retain_object(
            object_id,
            solid_object("far wall", bounds, material_id, |position| {
                position.z == depth - 1
            }),
        )
        .expect("an empty scene takes an object");

    scene
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .expect("the object is the scene's");

    scene
        .retain_view(RenderView {
            pose: TyPoseF64::new(
                TyVector3F64::new(f64::from(depth), f64::from(depth), 0.5),
                TyQuaternionF64::from_look_direction(TyVector3F64::Z, TyVector3F64::Y)
                    .expect("+Z and +Y are independent"),
            ),
            projection: RenderProjection::Perspective {
                fov: 90f64.to_radians(),
            },
        })
        .expect("a perspective view of 90 degrees is valid");

    scene
}

#[cfg(test)]
mod tests {
    use crate::{far_wall_scene, reference_hits};

    #[test]
    fn every_pixel_hits_the_wall() {
        let depth = 8;
        let scene = far_wall_scene(depth);
        let (view_id, _) = scene.iter_views().next().unwrap();

        for hit in reference_hits(&scene, view_id, 64).unwrap() {
            let hit = hit.unwrap();

            // Cells run along z fastest. The wall faces down z.
            assert_eq!(hit.voxel % depth, depth - 1);
            assert_eq!(hit.face, 4);
        }
    }
}
