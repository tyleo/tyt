use crate::{
    cell_noise, eye_view, outdoor_lights, painted_object, place_object, retain_matte,
    retain_next_object,
};
use ty_math::{
    TyPoseF64, TyQuaternionExt, TyQuaternionF64, TyVector3Ext, TyVector3F64, TyVector3U32,
};
use voxrender::{RenderProjection, RenderScene, RenderView};

/// The research's S5: a tiled stone floor of 2 cm voxels, 20.48 m square.
/// Seen from head height across its diagonal, where rays graze the tiles
/// toward the horizon, and from a low orthographic view.
pub fn grazing_floor_scene() -> RenderScene {
    let mut scene = RenderScene::default();

    let stones = [
        retain_matte(&mut scene, [0.55, 0.52, 0.48]),
        retain_matte(&mut scene, [0.45, 0.43, 0.4]),
    ];
    let grout = retain_matte(&mut scene, [0.25, 0.24, 0.22]);
    let pebble = retain_matte(&mut scene, [0.35, 0.3, 0.25]);

    let floor = retain_next_object(
        &mut scene,
        painted_object("floor", TyVector3U32::new(1024, 4, 1024), |p| {
            let joint = p.x % 32 == 0 || p.z % 32 == 0;
            let stone = stones[((p.x / 32 + p.z / 32) % 2) as usize];

            match p.y {
                0 => Some(grout),
                1 => Some(if joint { grout } else { stone }),
                2 => (!joint && cell_noise(31, p) < 0.85).then_some(stone),
                _ => (!joint && cell_noise(32, p) < 0.02).then_some(pebble),
            }
        }),
    );

    place_object(
        &mut scene,
        floor,
        TyVector3F64::ZERO,
        TyQuaternionF64::IDENTITY,
        0.02,
    );

    outdoor_lights(&mut scene, 60.0, 25.0);

    let eye = eye_view(
        TyVector3F64::new(0.6, 1.68, 0.6),
        TyVector3F64::new(20.4, 0.08, 20.4),
    );
    let from = TyVector3F64::from_azimuth_elevation(225f64.to_radians(), 10f64.to_radians());
    let low = RenderView {
        pose: TyPoseF64::new(
            TyVector3F64::new(10.24, 0.08, 10.24) + from * 30.0,
            TyQuaternionF64::from_look_direction(-from, TyVector3F64::Y)
                .expect("the view does not look straight down"),
        ),
        projection: RenderProjection::Orthographic { scale: 4.0 },
    };

    for view in [eye, low] {
        scene.retain_view(view).expect("a bench view is valid");
    }

    scene
}

#[cfg(test)]
mod tests {
    use crate::{grazing_floor_scene, reference_hits};

    #[test]
    fn every_view_sees_the_floor() {
        let scene = grazing_floor_scene();

        for (view_id, _) in scene.iter_views() {
            let hits = reference_hits(&scene, view_id, 32).unwrap();

            assert!(hits.iter().filter(|hit| hit.is_some()).count() * 10 > hits.len());
        }
    }
}
