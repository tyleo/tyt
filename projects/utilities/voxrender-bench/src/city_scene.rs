use crate::{
    cell_noise, eye_view, outdoor_lights, painted_object, place_object, retain_matte,
    retain_next_object,
};
use branded_id::U32Id;
use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{BRenderMaterial, RenderObject, RenderScene};

/// The blocks along each side of the city.
const BLOCKS: u32 = 16;

/// The meters from one street to the next.
const BLOCK_PITCH: f64 = 12.5;

/// The meters across a street.
const STREET: f64 = 3.0;

/// The research's S2: a city of 16 by 16 blocks in 1025 placements, with
/// buildings of 25 cm voxels, trees of 10 cm, and cars of 5 cm. Seen from a
/// street and from the air.
pub fn city_scene() -> RenderScene {
    let mut scene = RenderScene::default();

    let asphalt = retain_matte(&mut scene, [0.12, 0.12, 0.13]);
    let paving = retain_matte(&mut scene, [0.5, 0.5, 0.48]);
    let lawn = retain_matte(&mut scene, [0.2, 0.4, 0.15]);

    // Streets fill the first 6 of each block's 25 cells.
    let ground = retain_next_object(
        &mut scene,
        painted_object("ground", TyVector3U32::new(400, 1, 400), |p| {
            let block = p.x / 25 * BLOCKS + p.z / 25;

            Some(if p.x % 25 < 6 || p.z % 25 < 6 {
                asphalt
            } else if block.is_multiple_of(3) {
                lawn
            } else {
                paving
            })
        }),
    );

    let wall = retain_matte(&mut scene, [0.7, 0.66, 0.6]);
    let brick = retain_matte(&mut scene, [0.55, 0.25, 0.18]);
    let pane = retain_matte(&mut scene, [0.1, 0.14, 0.2]);
    let roof = retain_matte(&mut scene, [0.3, 0.2, 0.18]);

    let buildings = [
        retain_next_object(
            &mut scene,
            building("tower", TyVector3U32::new(28, 120, 28), wall, pane),
        ),
        retain_next_object(
            &mut scene,
            building("block", TyVector3U32::new(28, 56, 28), brick, pane),
        ),
        retain_next_object(&mut scene, house(wall, roof, pane)),
        retain_next_object(
            &mut scene,
            building("shed", TyVector3U32::new(16, 12, 20), brick, pane),
        ),
    ];

    let bark = retain_matte(&mut scene, [0.3, 0.2, 0.12]);
    let leaves = [
        retain_matte(&mut scene, [0.15, 0.35, 0.1]),
        retain_matte(&mut scene, [0.25, 0.45, 0.12]),
    ];

    let tree = retain_next_object(
        &mut scene,
        painted_object("tree", TyVector3U32::new(40, 70, 40), |p| {
            let dx = f64::from(p.x) - 19.5;
            let dz = f64::from(p.z) - 19.5;
            let dy = f64::from(p.y) - 45.0;
            let crown = (dx / 19.0).powi(2) + (dy / 24.0).powi(2) + (dz / 19.0).powi(2);

            if dx.abs() < 2.0 && dz.abs() < 2.0 && p.y < 40 {
                Some(bark)
            } else if crown < 1.0 && cell_noise(1, p) < 0.55 {
                Some(leaves[usize::from(cell_noise(2, p) < 0.5)])
            } else {
                None
            }
        }),
    );

    let tire = retain_matte(&mut scene, [0.05, 0.05, 0.05]);
    let paints = [[0.7, 0.1, 0.08], [0.1, 0.25, 0.6], [0.85, 0.85, 0.8]];
    let cars: Vec<_> = paints
        .into_iter()
        .map(|color| {
            let body = retain_matte(&mut scene, color);

            retain_next_object(&mut scene, car(body, pane, tire))
        })
        .collect();

    place_object(
        &mut scene,
        ground,
        TyVector3F64::new(0.0, -0.5, 0.0),
        TyQuaternionF64::IDENTITY,
        0.5,
    );

    for bx in 0..BLOCKS {
        for bz in 0..BLOCKS {
            let block = bx * BLOCKS + bz;
            let noise = |salt| cell_noise(salt, TyVector3U32::new(bx, 0, bz));
            let corner = TyVector3F64::new(f64::from(bx), 0.0, f64::from(bz)) * BLOCK_PITCH
                + TyVector3F64::new(STREET, 0.0, STREET);
            let center = corner + TyVector3F64::splat((BLOCK_PITCH - STREET) / 2.0);

            let building = buildings[(noise(3) * 4.0) as usize];
            let turn = if block % 5 == 4 {
                noise(4) * 360.0
            } else {
                (noise(4) * 4.0).floor() * 90.0
            };

            place_centered(
                &mut scene,
                building,
                center.with_y(0.0),
                turn,
                0.25 * (0.8 + 0.4 * noise(5)),
            );

            for (along, across) in [(0.1, 0.1), (0.8, 0.85)] {
                let at = corner + TyVector3F64::new(along, 0.0, across) * (BLOCK_PITCH - STREET);

                place_centered(&mut scene, tree, at, noise(6) * 360.0, 0.1);
            }

            // Cars park on the street at the block's -x side, facing along
            // it. Every ninth turns freely.
            let heading = if block % 9 == 8 {
                noise(7) * 360.0
            } else {
                (noise(7) * 2.0).floor() * 180.0 + noise(8) * 10.0 - 5.0
            };
            let at = corner + TyVector3F64::new(-STREET / 2.0, 0.0, noise(9) * 6.0 + 1.5);

            place_centered(&mut scene, cars[(block % 3) as usize], at, heading, 0.05);
        }
    }

    outdoor_lights(&mut scene, 120.0, 40.0);

    let middle = f64::from(BLOCKS) * BLOCK_PITCH / 2.0;
    let lane = 6.0 * BLOCK_PITCH + STREET / 2.0;
    let street = eye_view(
        TyVector3F64::new(lane, 1.6, 2.0),
        TyVector3F64::new(lane, 4.0, 100.0),
    );
    let air = eye_view(
        TyVector3F64::new(middle - 70.0, 90.0, middle - 100.0),
        TyVector3F64::new(middle, 0.0, middle),
    );

    for view in [street, air] {
        scene.retain_view(view).expect("a bench view is valid");
    }

    scene
}

/// A box of `bounds` with walls two cells thick of `wall`, and windows of
/// `pane` set one cell into them.
fn building(
    name: &str,
    bounds: TyVector3U32,
    wall: U32Id<BRenderMaterial>,
    pane: U32Id<BRenderMaterial>,
) -> RenderObject {
    painted_object(name, bounds, |p| {
        let outer = p.x == 0 || p.x == bounds.x - 1 || p.z == 0 || p.z == bounds.z - 1;
        let side = !(2..bounds.x - 2).contains(&p.x);
        let along = if side { p.z } else { p.x };
        let window = (2..6).contains(&(p.y % 8)) && (1..4).contains(&(along % 6));
        let shell = side || !(2..bounds.z - 2).contains(&p.z) || p.y >= bounds.y - 2;

        if outer && window && p.y < bounds.y - 2 {
            None
        } else if shell && window && p.y < bounds.y - 2 {
            Some(pane)
        } else {
            shell.then_some(wall)
        }
    })
}

/// A house of `wall` with windows of `pane` under a gabled roof of `roof`.
fn house(
    wall: U32Id<BRenderMaterial>,
    roof: U32Id<BRenderMaterial>,
    pane: U32Id<BRenderMaterial>,
) -> RenderObject {
    let body = building("house body", TyVector3U32::new(24, 18, 24), wall, pane);

    painted_object("house", TyVector3U32::new(24, 30, 24), |p| {
        if p.y < 18 {
            let voxel_id = body.voxel_id(p).expect("the body spans the house");

            return body.voxel_material(voxel_id);
        }

        let rise = f64::from(p.y - 18);
        let reach = 12.0 - rise - (f64::from(p.z) - 11.5).abs();

        if (0.0..2.0).contains(&reach) {
            Some(roof)
        } else if reach > 0.0 && (p.x < 2 || p.x >= 22) {
            Some(wall)
        } else {
            None
        }
    })
}

/// A car of `body` with windows of `pane` and wheels of `tire`, facing +z.
fn car(
    body: U32Id<BRenderMaterial>,
    pane: U32Id<BRenderMaterial>,
    tire: U32Id<BRenderMaterial>,
) -> RenderObject {
    painted_object("car", TyVector3U32::new(36, 28, 80), |p| {
        let hub = |z: f64| (f64::from(p.y) - 7.0).hypot(f64::from(p.z) - z) < 7.0;
        let cabin = (14..26).contains(&p.y) && (4..32).contains(&p.x) && (22..58).contains(&p.z);
        let glass = (17..24).contains(&p.y) && (p.x == 4 || p.x == 31 || p.z == 22 || p.z == 57);

        if (p.x < 4 || p.x >= 32) && (hub(14.0) || hub(66.0)) {
            Some(tire)
        } else if (4..14).contains(&p.y) && (1..35).contains(&p.x) {
            Some(body)
        } else if cabin && glass {
            Some(pane)
        } else {
            cabin.then_some(body)
        }
    })
}

/// Places `object_id` with its grid's base center at `at`, turned
/// `degrees` about +y, at `scale` meters per voxel.
fn place_centered(
    scene: &mut RenderScene,
    object_id: U32Id<BVoxObject>,
    at: TyVector3F64,
    degrees: f64,
    scale: f64,
) {
    let bounds = scene
        .object(object_id)
        .expect("the object is the scene's")
        .bounds()
        .as_dvec3();
    let rotation = TyQuaternionF64::from_rotation_y(degrees.to_radians());
    let half = TyVector3F64::new(bounds.x, 0.0, bounds.z) * scale / 2.0;

    place_object(scene, object_id, at - rotation * half, rotation, scale);
}

#[cfg(test)]
mod tests {
    use crate::{city_scene, reference_hits};

    #[test]
    fn the_city_holds_a_thousand_placements() {
        assert!(city_scene().placement_count() >= 1000);
    }

    #[test]
    fn every_view_sees_the_city() {
        let scene = city_scene();

        for (view_id, _) in scene.iter_views() {
            let hits = reference_hits(&scene, view_id, 32).unwrap();

            assert!(hits.iter().filter(|hit| hit.is_some()).count() * 10 > hits.len());
        }
    }
}
