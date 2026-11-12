// The contract's bloom over a `BoxView`'s eyes, in passes that each draw one
// triangle over both eyes. The view draws the source: the emission over the
// threshold at half the eyes' size. `down` halves the source into the first
// level and each level into the next. `up` adds a level into the one before
// it. The first level then holds the halo the eyes' fragments add.

struct Bloom {
    // What `up` scales the level it adds by.
    scale: f32,
}

struct Corner {
    @builtin(position) clip: vec4<f32>,

    // The point across the target, from its top left.
    @location(0) across: vec2<f32>,

    @location(1) @interpolate(flat) eye: u32,
}

@group(0) @binding(0) var<uniform> bloom: Bloom;
// The level a pass reads.
@group(0) @binding(1) var read: texture_2d_array<f32>;
@group(0) @binding(2) var read_sampler: sampler;

@vertex
fn corner(@builtin(vertex_index) vertex: u32, @builtin(view_index) eye: u32) -> Corner {
    let across = vec2<f32>(f32(vertex & 1u), f32(vertex >> 1u)) * 2.0;

    return Corner(vec4<f32>(across.x * 2.0 - 1.0, 1.0 - across.y * 2.0, 0.0, 1.0), across, eye);
}

// The level under the fragment through a tent 4 of its texels wide: four
// taps, each between four texels.
@fragment
fn down(corner: Corner) -> @location(0) vec4<f32> {
    let texel = 0.75 / vec2<f32>(textureDimensions(read));
    var sum = vec3<f32>(0.0);

    for (var tap = 0u; tap < 4u; tap++) {
        let offset = (vec2<f32>(f32(tap & 1u), f32(tap >> 1u)) * 2.0 - 1.0) * texel;

        sum += textureSampleLevel(read, read_sampler, corner.across + offset, corner.eye, 0.0).rgb;
    }

    return vec4<f32>(sum / 4.0, 1.0);
}

// The coarser level under the fragment, which the target's blend adds to
// what the target holds.
@fragment
fn up(corner: Corner) -> @location(0) vec4<f32> {
    return vec4<f32>(
        textureSampleLevel(read, read_sampler, corner.across, corner.eye, 0.0).rgb * bloom.scale,
        1.0,
    );
}
