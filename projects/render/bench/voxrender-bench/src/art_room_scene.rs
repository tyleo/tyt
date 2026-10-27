use crate::{
    eye_view, fitted_view, outdoor_lights, painted_object, place_object, retain_matte,
    retain_next_object,
};
use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
use voxrender::RenderScene;

/// The research's S1: a voxel-art room of 2 cm voxels, open on two sides,
/// furnished with 1 cm props and seen from arm's length over its table. The
/// sun comes in through a window and over the open sides.
pub fn art_room_scene() -> RenderScene {
    let mut scene = RenderScene::default();

    let plaster = retain_matte(&mut scene, [0.85, 0.82, 0.75]);
    let baseboard = retain_matte(&mut scene, [0.3, 0.25, 0.2]);
    let planks = [
        retain_matte(&mut scene, [0.55, 0.36, 0.2]),
        retain_matte(&mut scene, [0.48, 0.3, 0.16]),
    ];

    let room = retain_next_object(
        &mut scene,
        painted_object("room", TyVector3U32::new(150, 130, 150), |p| {
            if p.y < 2 {
                // Rows of planks 8 cells wide, each row's joints offset.
                let row = p.z / 8;
                let board = (p.x + row * 37 % 60) / 60;

                return Some(planks[((row + board) % 2) as usize]);
            }

            let window = p.z < 2 && (60..100).contains(&p.x) && (45..95).contains(&p.y);

            if (p.x >= 2 && p.z >= 2) || window {
                None
            } else if p.y < 6 {
                Some(baseboard)
            } else {
                Some(plaster)
            }
        }),
    );

    let wood = retain_matte(&mut scene, [0.4, 0.25, 0.12]);

    let table = retain_next_object(
        &mut scene,
        painted_object("table", TyVector3U32::new(80, 76, 60), |p| {
            let leg = (p.x < 5 || p.x >= 75) && (p.z < 5 || p.z >= 55);
            let apron = p.y >= 64 && (p.x < 5 || p.x >= 75 || p.z < 5 || p.z >= 55);

            (p.y >= 72 || leg || apron).then_some(wood)
        }),
    );

    let glaze = [
        retain_matte(&mut scene, [0.2, 0.35, 0.7]),
        retain_matte(&mut scene, [0.9, 0.85, 0.7]),
    ];

    let vase = retain_next_object(
        &mut scene,
        painted_object("vase", TyVector3U32::new(21, 30, 21), |p| {
            let radius = 5.0 + 3.0 * (f64::from(p.y) * 0.25).sin();
            let dx = f64::from(p.x) - 10.0;
            let dz = f64::from(p.z) - 10.0;
            let distance = (dx * dx + dz * dz).sqrt();

            (distance < radius && (distance > radius - 2.0 || p.y < 2))
                .then_some(glaze[(p.y / 4 % 2) as usize])
        }),
    );

    let covers = [
        retain_matte(&mut scene, [0.6, 0.12, 0.1]),
        retain_matte(&mut scene, [0.15, 0.3, 0.2]),
        retain_matte(&mut scene, [0.75, 0.6, 0.25]),
        retain_matte(&mut scene, [0.12, 0.15, 0.35]),
    ];

    let books = retain_next_object(
        &mut scene,
        painted_object("books", TyVector3U32::new(32, 24, 16), |p| {
            let book = p.x / 4;

            (p.x % 4 != 3 && p.y < 18 + book * 5 % 7).then_some(covers[(book % 4) as usize])
        }),
    );

    let frame = retain_matte(&mut scene, [0.35, 0.22, 0.1]);
    let board = retain_matte(&mut scene, [0.62, 0.45, 0.25]);

    let crate_object = retain_next_object(
        &mut scene,
        painted_object("crate", TyVector3U32::new(30, 30, 30), |p| {
            let rims = [p.x, p.y, p.z].map(|c| !(3..27).contains(&c));
            let faces = [p.x, p.y, p.z].map(|c| !(2..28).contains(&c));

            if rims.iter().filter(|&&rim| rim).count() >= 2 {
                Some(frame)
            } else if faces[1] || ((faces[0] || faces[2]) && p.y % 6 != 5) {
                Some(board)
            } else {
                None
            }
        }),
    );

    let threads = [
        retain_matte(&mut scene, [0.6, 0.1, 0.1]),
        retain_matte(&mut scene, [0.9, 0.75, 0.3]),
        retain_matte(&mut scene, [0.1, 0.2, 0.45]),
    ];

    let rug = retain_next_object(
        &mut scene,
        painted_object("rug", TyVector3U32::new(160, 1, 100), |p| {
            let border = p.x < 8 || p.x >= 152 || p.z < 8 || p.z >= 92;

            Some(if border {
                threads[0]
            } else {
                threads[((p.x / 10 + p.z / 10) % 3) as usize]
            })
        }),
    );

    let floor = 0.04;
    let top = floor + 0.76;
    let level = TyQuaternionF64::IDENTITY;
    let turned = TyQuaternionF64::from_rotation_y;

    place_object(&mut scene, room, TyVector3F64::ZERO, level, 0.02);
    place_object(
        &mut scene,
        rug,
        TyVector3F64::new(1.0, floor, 0.9),
        level,
        0.01,
    );
    place_object(
        &mut scene,
        table,
        TyVector3F64::new(1.4, floor, 1.2),
        level,
        0.01,
    );
    place_object(
        &mut scene,
        vase,
        TyVector3F64::new(1.55, top, 1.35),
        level,
        0.01,
    );
    place_object(
        &mut scene,
        books,
        TyVector3F64::new(1.85, top, 1.25),
        turned(15f64.to_radians()),
        0.01,
    );

    for (position, degrees) in [
        (TyVector3F64::new(0.1, floor, 0.1), 0.0),
        (TyVector3F64::new(0.1, floor + 0.3, 0.12), 20.0),
        (TyVector3F64::new(0.45, floor, 0.1), -10.0),
    ] {
        place_object(
            &mut scene,
            crate_object,
            position,
            turned(f64::to_radians(degrees)),
            0.01,
        );
    }

    outdoor_lights(&mut scene, 180.0, 35.0);

    let eye = eye_view(
        TyVector3F64::new(1.8, 1.5, 2.35),
        TyVector3F64::new(1.75, top, 1.45),
    );
    let fitted = fitted_view(&scene, 45.0, 35.0);

    for view in [eye, fitted] {
        scene.retain_view(view).expect("a bench view is valid");
    }

    scene
}

#[cfg(test)]
mod tests {
    use crate::{art_room_scene, reference_hits};

    #[test]
    fn every_view_sees_the_room() {
        let scene = art_room_scene();

        for (view_id, _) in scene.iter_views() {
            let hits = reference_hits(&scene, view_id, 32).unwrap();

            assert!(hits.iter().filter(|hit| hit.is_some()).count() * 10 > hits.len());
        }
    }
}
