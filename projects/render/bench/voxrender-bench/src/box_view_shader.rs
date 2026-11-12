use crate::TileMasks;

/// The WGSL of [`BoxView`](crate::BoxView)'s shader module, for a tool that
/// compiles it apart from the view.
///
/// Its vertex entries take a vertex's bits at location 0 and a box's three
/// `vec4<u32>`s at locations 1 to 3: `place` and `place_near` for a box's
/// back faces, and `place_front` for its faces toward the view. Its
/// fragment entries are `front`, `crossed`, `inside`, `exact`, and `solid`.
/// Group 0 binds a uniform buffer at 0, read-only storage buffers at 1 to
/// 6, a two-layer texture of filterable floats at 7, and its sampler at 8.
pub fn box_view_shader() -> String {
    format!(
        "const RECORDS: u32 = {}u;\n{}\n{}",
        TileMasks::RECORDS,
        include_str!("box_view.wgsl"),
        include_str!("mask_march.wgsl")
    )
}
