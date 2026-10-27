use crate::{
    cell_noise, eye_view, outdoor_lights, painted_object, place_object, retain_matte,
    retain_next_object,
};
use branded_id::U32Id;
use ty_math::{TyLinSrgbaF64, TyQuaternionF64, TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{RenderScene, fixtures::glass};

/// The meters per voxel of the greenhouse's structure.
const SCALE: f64 = 0.02;

/// The research's S3: a greenhouse of 2 cm voxels, 6 by 3 by 4 m, with
/// plants of 1 cm voxels inside. Each of its 84 glass panes overlaps its
/// neighbors. Seen from outside through two walls, from inside, and from
/// the air.
pub fn greenhouse_scene() -> RenderScene {
    let mut scene = RenderScene::default();

    let tiles = [
        retain_matte(&mut scene, [0.45, 0.42, 0.38]),
        retain_matte(&mut scene, [0.35, 0.33, 0.3]),
    ];

    let floor = retain_next_object(
        &mut scene,
        painted_object("floor", TyVector3U32::new(340, 2, 240), |p| {
            Some(tiles[((p.x / 20 + p.z / 20) % 2) as usize])
        }),
    );

    let steel = retain_matte(&mut scene, [0.3, 0.32, 0.35]);

    let frame = retain_next_object(
        &mut scene,
        painted_object("frame", TyVector3U32::new(300, 150, 200), |p| {
            let side = !(3..297).contains(&p.x);
            let end = !(3..197).contains(&p.z);
            let roof = p.y >= 147;
            let edges = [side, roof || p.y < 3, end]
                .into_iter()
                .filter(|&edge| edge)
                .count();
            let post = (end && p.x % 50 < 3) || (side && p.z % 50 < 3);
            let rail = (side || end) && p.y % 75 < 3;
            let rafter = roof && p.x % 50 < 3;

            (edges >= 2 || post || rail || rafter).then_some(steel)
        }),
    );

    let panes = [
        TyLinSrgbaF64::new(0.85, 0.95, 0.9, 1.0),
        TyLinSrgbaF64::new(0.6, 0.85, 0.8, 1.0),
    ]
    .map(|tint| {
        let material_id = scene
            .retain_material(glass(tint))
            .expect("a glass tint is a valid material");

        retain_next_object(
            &mut scene,
            painted_object("pane", TyVector3U32::new(56, 56, 1), |_| Some(material_id)),
        )
    });

    let wood = retain_matte(&mut scene, [0.45, 0.3, 0.18]);

    let bench = retain_next_object(
        &mut scene,
        painted_object("bench", TyVector3U32::new(250, 40, 50), |p| {
            let leg = (p.x < 3 || p.x >= 247) && (p.z < 3 || p.z >= 47);

            (p.y >= 36 || leg).then_some(wood)
        }),
    );

    let clay = retain_matte(&mut scene, [0.65, 0.32, 0.2]);
    let soil = retain_matte(&mut scene, [0.2, 0.13, 0.08]);
    let leaves = [
        retain_matte(&mut scene, [0.12, 0.4, 0.12]),
        retain_matte(&mut scene, [0.3, 0.55, 0.15]),
    ];

    let plants = [11, 12].map(|seed| {
        retain_next_object(
            &mut scene,
            painted_object("plant", TyVector3U32::new(24, 50, 24), |p| {
                let dx = f64::from(p.x) - 11.5;
                let dz = f64::from(p.z) - 11.5;
                let distance = dx.hypot(dz);
                let radius = 7.0 + f64::from(p.y) * 0.2;
                let dy = f64::from(p.y) - 36.0;
                let crown = (distance / 11.0).powi(2) + (dy / 13.0).powi(2);

                if p.y < 16 && distance < radius && (distance > radius - 2.0 || p.y < 2) {
                    Some(clay)
                } else if (13..16).contains(&p.y) && distance < radius {
                    Some(soil)
                } else if p.y >= 16 && p.y < 30 && dx.abs() < 1.0 && dz.abs() < 1.0 {
                    Some(leaves[0])
                } else if crown < 1.0 && cell_noise(seed, p) < 0.4 {
                    Some(leaves[usize::from(cell_noise(seed + 1, p) < 0.5)])
                } else {
                    None
                }
            }),
        )
    });

    let level = TyQuaternionF64::IDENTITY;

    place_object(
        &mut scene,
        floor,
        TyVector3F64::new(-0.4, -0.04, -0.4),
        level,
        SCALE,
    );
    place_object(&mut scene, frame, TyVector3F64::ZERO, level, SCALE);

    let facing_x = TyQuaternionF64::from_rotation_y(90f64.to_radians());
    let mut count = 0;

    let mut glaze = |scene: &mut RenderScene, center: TyVector3F64, rotation| {
        place_pane(scene, panes[usize::from(count % 4 == 3)], center, rotation);
        count += 1;
    };

    for i in 0..6 {
        for j in 0..3 {
            let out = 0.01 + 0.02 * f64::from((i + j) % 2);
            let (x, y) = (f64::from(i) + 0.5, f64::from(j) + 0.5);

            glaze(&mut scene, TyVector3F64::new(x, y, -out), level);
            glaze(&mut scene, TyVector3F64::new(x, y, 4.0 + out), level);
        }
    }

    for k in 0..4 {
        for j in 0..3 {
            let out = 0.01 + 0.02 * f64::from((k + j) % 2);
            let (y, z) = (f64::from(j) + 0.5, f64::from(k) + 0.5);

            glaze(&mut scene, TyVector3F64::new(-out, y, z), facing_x);
            glaze(&mut scene, TyVector3F64::new(6.0 + out, y, z), facing_x);
        }
    }

    for i in 0..6 {
        for k in 0..4 {
            let alternate = (i + k) % 2;
            let tilt = if alternate == 0 { 93.0 } else { 87.0 };
            let center = TyVector3F64::new(
                f64::from(i) + 0.5,
                3.03 + 0.02 * f64::from(alternate),
                f64::from(k) + 0.5,
            );

            glaze(
                &mut scene,
                center,
                TyQuaternionF64::from_rotation_x(f64::to_radians(tilt)),
            );
        }
    }

    for z in [0.4, 2.6] {
        place_object(
            &mut scene,
            bench,
            TyVector3F64::new(0.5, 0.0, z),
            level,
            SCALE,
        );

        for k in 0..12 {
            let at = TyVector3F64::new(
                0.6 + 0.38 * f64::from(k),
                0.8,
                z + 0.3 + 0.2 * f64::from(k % 2),
            );
            let turn = cell_noise(13, TyVector3U32::new(k, 0, (z * 10.0) as u32)) * 360.0;

            place_object(
                &mut scene,
                plants[(k % 2) as usize],
                at,
                TyQuaternionF64::from_rotation_y(turn.to_radians()),
                0.01,
            );
        }
    }

    outdoor_lights(&mut scene, 200.0, 50.0);

    let views = [
        eye_view(
            TyVector3F64::new(1.2, 1.6, -2.2),
            TyVector3F64::new(3.4, 1.0, 2.0),
        ),
        eye_view(
            TyVector3F64::new(5.4, 1.6, 2.0),
            TyVector3F64::new(0.6, 1.1, 0.9),
        ),
        eye_view(
            TyVector3F64::new(-1.0, 5.0, -1.5),
            TyVector3F64::new(3.0, 0.5, 2.0),
        ),
    ];

    for view in views {
        scene.retain_view(view).expect("a bench view is valid");
    }

    scene
}

/// Places `pane` with its center at `center`, turned by `rotation`.
fn place_pane(
    scene: &mut RenderScene,
    pane: U32Id<BVoxObject>,
    center: TyVector3F64,
    rotation: TyQuaternionF64,
) {
    let half = scene
        .object(pane)
        .expect("the pane is the scene's")
        .bounds()
        .as_dvec3()
        * SCALE
        / 2.0;

    place_object(scene, pane, center - rotation * half, rotation, SCALE);
}

#[cfg(test)]
mod tests {
    use crate::{greenhouse_scene, reference_hits};

    #[test]
    fn the_greenhouse_has_84_panes() {
        // The floor, the frame, two benches, and 24 plants.
        assert_eq!(greenhouse_scene().placement_count(), 84 + 28);
    }

    #[test]
    fn every_view_sees_the_greenhouse() {
        let scene = greenhouse_scene();

        for (view_id, _) in scene.iter_views() {
            let hits = reference_hits(&scene, view_id, 32).unwrap();

            assert!(hits.iter().filter(|hit| hit.is_some()).count() * 10 > hits.len());
        }
    }
}
