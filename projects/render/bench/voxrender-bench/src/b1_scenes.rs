use crate::{art_room_scene, city_scene, foliage_scene, grazing_floor_scene, greenhouse_scene};
use voxrender::{
    RenderScene, RenderShadow,
    fixtures::{
        cube_scene, glass_scene, glow_scene, l_shape_scene, precision_shadow_scene,
        precision_view_scene, random_scene, room_scene, spot_room_scene, stairs_scene,
        two_placements_scene,
    },
};

/// The golden, precision-sim, and bench scenes B1 runs over, each with the
/// name its report gives it.
pub fn b1_scenes() -> Vec<(&'static str, RenderScene)> {
    vec![
        ("cube", cube_scene(RenderShadow::PerPixel)),
        ("glass", glass_scene(RenderShadow::PerPixel)),
        ("glow", glow_scene(RenderShadow::PerPixel)),
        ("l-shape", l_shape_scene(RenderShadow::PerPixel)),
        ("room", room_scene(RenderShadow::PerPixel)),
        ("spot-room", spot_room_scene(RenderShadow::PerPixel)),
        (
            "two-placements",
            two_placements_scene(RenderShadow::PerPixel),
        ),
        ("precision views", precision_view_scene()),
        (
            "precision shadows, sparse",
            precision_shadow_scene(random_scene(1, 0.15, 1)),
        ),
        (
            "precision shadows, dense",
            precision_shadow_scene(random_scene(2, 0.40, 1)),
        ),
        (
            "precision shadows, stairs",
            precision_shadow_scene(stairs_scene()),
        ),
        ("S1 art room", art_room_scene()),
        ("S2 city", city_scene()),
        ("S3 greenhouse", greenhouse_scene()),
        ("S4 foliage", foliage_scene()),
        ("S5 grazing floor", grazing_floor_scene()),
    ]
}
