use crate::b1_scenes;
use voxrender::RenderScene;

/// The first of B1's scenes whose name starts with `name`, such as `S1`.
pub fn bench_scene(name: &str) -> Option<RenderScene> {
    b1_scenes()
        .into_iter()
        .find(|(scene_name, _)| scene_name.starts_with(name))
        .map(|(_, scene)| scene)
}
