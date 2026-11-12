use crate::{BoxView, TileMasks};

/// What stands in for `box_mirror.wgsl` in the module with no ray query,
/// where no face casts a mirror ray.
const UNTRACED: &str = "
fn mirrored(origin: vec3<f32>, direction: vec3<f32>, reach: f32) -> vec4<f32> {
    return vec4<f32>(-1.0);
}
";

/// The WGSL of [`BoxView`](crate::BoxView)'s shader module, for a tool that
/// compiles it apart from the view. With `traced`, a face's mirror ray
/// runs a ray query, which needs a device with ray queries. Without
/// `traced`, no face casts one.
///
/// Its vertex entries take a vertex's bits at location 0 and a box's three
/// `vec4<u32>`s at locations 1 to 3: `place`, `place_near`, and
/// `place_held` for a box's back faces, and `place_front` for its faces
/// toward the view. Its fragment entries are `front`, `crossed`, `inside`,
/// `held`, `exact`, and `solid`. Group 0 binds a uniform buffer at 0,
/// read-only storage buffers at 1 to 7, a two-layer texture of filterable
/// floats at 8, its sampler at 9, and with `traced` an acceleration
/// structure at 10.
pub fn box_view_shader(traced: bool) -> String {
    format!(
        "{}const RECORDS: u32 = {}u;\nconst MIRRORS: bool = {traced};\n\
         const MIRROR_CUT: f32 = {:?};\n{}\n{}\n{}",
        if traced {
            "enable wgpu_ray_query;\n"
        } else {
            ""
        },
        TileMasks::RECORDS,
        BoxView::MIRROR_CUT,
        include_str!("box_view.wgsl"),
        if traced {
            include_str!("box_mirror.wgsl")
        } else {
            UNTRACED
        },
        include_str!("mask_march.wgsl")
    )
}
