use crate::{
    RenderPlacement, RenderProjection, RenderScene, RenderView,
    fixtures::{matte, orbit_view, precision_lights, solid_object},
};
use branded_id::U32Id;
use ty_math::{
    TyLinSrgbaF64, TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3F64,
    TyVector3U32,
};

/// `cells`, a 24 by 10 by 24 precision scene whose bottom layer is empty,
/// over a floor that fills that layer as a second placement. The
/// [precision lights](precision_lights) light it, and it is seen from
/// straight above and from two orbits. The floor's top faces start the
/// precision-sim's corner shadow rays.
pub fn precision_shadow_scene(mut cells: RenderScene) -> RenderScene {
    let material_id = cells
        .retain_material(matte(TyLinSrgbaF64::new(0.5, 0.5, 0.5, 1.0)))
        .unwrap();

    let object_id = U32Id::from_u32(1);

    cells
        .retain_object(
            object_id,
            solid_object("floor", TyVector3U32::new(24, 1, 24), material_id, |_| true),
        )
        .unwrap();

    cells
        .retain_placement(RenderPlacement {
            object_id,
            transform: TyTransformF64::IDENTITY,
        })
        .unwrap();

    for light in precision_lights() {
        cells.retain_light(light).unwrap();
    }

    let center = TyVector3F64::new(12.0, 1.0, 12.0);

    cells
        .retain_view(RenderView {
            pose: TyPoseF64::new(
                center + TyVector3F64::Y * 40.0,
                TyQuaternionF64::from_look_direction(-TyVector3F64::Y, -TyVector3F64::Z).unwrap(),
            ),
            projection: RenderProjection::Orthographic { scale: 24.0 },
        })
        .unwrap();

    for (azimuth, elevation) in [(45.0, 30.0), (-30.0, 60.0)] {
        cells
            .retain_view(orbit_view(center, azimuth, elevation, 40.0))
            .unwrap();
    }

    cells
}
