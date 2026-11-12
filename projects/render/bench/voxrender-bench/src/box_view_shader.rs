use crate::{BoxView, TileMasks};

/// What stands in for `box_mirror.wgsl` in the module with no ray query,
/// where no ray is cast.
const UNTRACED: &str = "
fn mirrored(origin: vec3<f32>, direction: vec3<f32>, reach: f32) -> vec4<f32> {
    return vec4<f32>(-1.0);
}

fn hidden(origin: vec3<f32>, direction: vec3<f32>, reach: f32) -> bool {
    return false;
}
";

/// The WGSL of [`BoxView`](crate::BoxView)'s shader module, for a tool that
/// compiles it apart from the view.
///
/// Its vertex entries take a vertex's bits at location 0 and a box's three
/// `vec4<u32>`s at locations 1 to 3: `place`, `place_near`, and
/// `place_held` for a box's back faces, and `place_front` for its faces
/// toward the view. Its fragment entries are `front`, `crossed`, `inside`,
/// `held`, `exact`, and `solid`. `place_sky` and `sky_halo` draw the halo
/// over the sky. Group 0 binds a uniform buffer at 0, read-only storage
/// buffers at 1 to 7, a two-layer texture of filterable floats at 8, its
/// sampler at 9, and with `traced` an acceleration structure at 10.
///
/// # Arguments
///
/// 1. `traced`: whether a face's mirror ray runs a ray query, which a device
///    needs ray queries for. Without it, no face casts one
/// 2. `bloom`: the level of an emission's brightest channel that blooms.
///    With it, a fragment adds the halo
/// 3. `source`: with it, the module draws the bloom's source in place of the
///    view: a hit's emission over the `bloom` level times `source`, or
///    nothing where a quad hides the hit from the eye. It takes `traced`
pub fn box_view_shader(traced: bool, bloom: Option<f32>, source: Option<f32>) -> String {
    format!(
        "{}const RECORDS: u32 = {}u;\nconst MIRRORS: bool = {traced};\n\
         const MIRROR_CUT: f32 = {:?};\nconst MIRROR_REACH: f32 = {:?};\n\
         const BLOOM: bool = {};\n\
         const BLOOM_THRESHOLD: f32 = {:?};\nconst BLOOM_SOURCE: f32 = {:?};\n{}\n{}\n{}",
        if traced {
            "enable wgpu_ray_query;\n"
        } else {
            ""
        },
        TileMasks::RECORDS,
        BoxView::MIRROR_CUT,
        BoxView::MIRROR_REACH,
        bloom.is_some(),
        bloom.unwrap_or(0.0),
        source.unwrap_or(0.0),
        include_str!("box_view.wgsl"),
        if traced {
            include_str!("box_mirror.wgsl")
        } else {
            UNTRACED
        },
        include_str!("mask_march.wgsl")
    )
}
