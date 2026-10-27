use crate::{
    cell_noise, eye_view, outdoor_lights, painted_object, place_object, retain_matte,
    retain_next_object,
};
use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
use voxrender::RenderScene;

/// The meters per voxel.
const SCALE: f64 = 0.02;

/// The research's S4, the silhouette worst case: a garden of 2 cm voxels
/// with picket fences one voxel wide on one-voxel gaps, sparse bushes, and
/// trees with sparse crowns. Seen along a fence and from under a tree
/// against the sky.
pub fn foliage_scene() -> RenderScene {
    let mut scene = RenderScene::default();

    let grass = [
        retain_matte(&mut scene, [0.2, 0.4, 0.12]),
        retain_matte(&mut scene, [0.25, 0.45, 0.15]),
    ];

    let ground = retain_next_object(
        &mut scene,
        painted_object("ground", TyVector3U32::new(500, 1, 500), |p| {
            Some(grass[usize::from(cell_noise(21, p) < 0.5)])
        }),
    );

    let paint = retain_matte(&mut scene, [0.85, 0.85, 0.8]);

    let fence = retain_next_object(
        &mut scene,
        painted_object("fence", TyVector3U32::new(100, 60, 3), |p| {
            let post = p.x < 3 || p.x >= 97;
            let rail = p.z < 2 && ((12..15).contains(&p.y) || (44..47).contains(&p.y));
            let picket = p.z == 2 && p.x % 2 == 0;

            (post || rail || picket).then_some(paint)
        }),
    );

    let leaves = [
        retain_matte(&mut scene, [0.1, 0.3, 0.08]),
        retain_matte(&mut scene, [0.2, 0.42, 0.1]),
    ];

    let bush = retain_next_object(
        &mut scene,
        painted_object("bush", TyVector3U32::new(60, 46, 60), |p| {
            let shape = ((f64::from(p.x) - 29.5) / 29.5).powi(2)
                + ((f64::from(p.y) - 20.0) / 22.0).powi(2)
                + ((f64::from(p.z) - 29.5) / 29.5).powi(2);

            (shape < 1.0 && cell_noise(22, p) < 0.3)
                .then(|| leaves[usize::from(cell_noise(23, p) < 0.5)])
        }),
    );

    let bark = retain_matte(&mut scene, [0.3, 0.2, 0.12]);

    let tree = retain_next_object(
        &mut scene,
        painted_object("tree", TyVector3U32::new(90, 160, 90), |p| {
            let dx = f64::from(p.x) - 44.5;
            let dz = f64::from(p.z) - 44.5;
            let crown = (dx / 44.5).powi(2)
                + ((f64::from(p.y) - 110.0) / 48.0).powi(2)
                + (dz / 44.5).powi(2);

            if dx.abs() < 4.0 && dz.abs() < 4.0 && p.y < 130 {
                Some(bark)
            } else if crown < 1.0 && cell_noise(24, p) < 0.18 {
                Some(leaves[usize::from(cell_noise(25, p) < 0.5)])
            } else {
                None
            }
        }),
    );

    place_object(
        &mut scene,
        ground,
        TyVector3F64::new(0.0, -SCALE, 0.0),
        TyQuaternionF64::IDENTITY,
        SCALE,
    );

    let runs = [
        (TyVector3F64::new(1.0, 0.0, 3.0), 0.0, 5),
        (TyVector3F64::new(1.0, 0.0, 3.0), -90.0, 3),
        (TyVector3F64::new(5.0, 0.0, 6.0), 33.0, 4),
    ];

    for (start, degrees, sections) in runs {
        let rotation = TyQuaternionF64::from_rotation_y(f64::to_radians(degrees));
        let along = rotation * TyVector3F64::X * 100.0 * SCALE;

        for section in 0..sections {
            place_object(
                &mut scene,
                fence,
                start + along * f64::from(section),
                rotation,
                SCALE,
            );
        }
    }

    let bushes = [
        (2.0, 5.0),
        (3.5, 6.5),
        (5.0, 4.5),
        (7.0, 7.5),
        (8.5, 5.0),
        (2.5, 8.5),
        (6.0, 9.0),
        (9.0, 8.5),
    ];

    for (k, (x, z)) in (0..).zip(bushes) {
        let turn = cell_noise(26, TyVector3U32::new(k, 0, 0)) * 360.0;

        place_object(
            &mut scene,
            bush,
            TyVector3F64::new(x, 0.0, z),
            TyQuaternionF64::from_rotation_y(turn.to_radians()),
            SCALE,
        );
    }

    for (x, z) in [(3.1, 7.1), (7.2, 1.2), (8.0, 6.0)] {
        place_object(
            &mut scene,
            tree,
            TyVector3F64::new(x, 0.0, z),
            TyQuaternionF64::IDENTITY,
            SCALE,
        );
    }

    outdoor_lights(&mut scene, 240.0, 45.0);

    // The first tree's trunk stands at (4, 8).
    let views = [
        eye_view(
            TyVector3F64::new(0.2, 1.4, 2.5),
            TyVector3F64::new(10.0, 0.9, 3.4),
        ),
        eye_view(
            TyVector3F64::new(4.2, 0.4, 6.8),
            TyVector3F64::new(4.0, 3.0, 8.0),
        ),
    ];

    for view in views {
        scene.retain_view(view).expect("a bench view is valid");
    }

    scene
}

#[cfg(test)]
mod tests {
    use crate::{foliage_scene, reference_hits};

    #[test]
    fn every_view_sees_the_garden() {
        let scene = foliage_scene();

        for (view_id, _) in scene.iter_views() {
            let hits = reference_hits(&scene, view_id, 32).unwrap();

            assert!(hits.iter().filter(|hit| hit.is_some()).count() * 10 > hits.len());
        }
    }
}
