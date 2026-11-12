// The ids a mask march writes, over `mask_march.wgsl`: a raster of the boxes
// around each tile's live cells over `TileMasks`. A hit packs as
// `raster.wgsl`'s does. `mask_boxes` prepends `FACE_BITS` and `RAY_W`.

struct Frame {
    // The low bits of a packed hit that hold its voxel id.
    voxel_bits: u32,
}

struct Place {
    // The placement's grid-to-clip matrix.
    clip: mat4x4<f32>,

    // A perspective view's eye in the grid with w = 1, or the reverse of an
    // orthographic view's direction in the grid with w = 0.
    ray: vec4<f32>,

    // What a step along x and a step along y add to a voxel id.
    strides: vec4<u32>,

    // The world direction of each of the grid's axes, then the meters a cell
    // spans along it.
    x: vec4<f32>,
    y: vec4<f32>,
    z: vec4<f32>,
}

// Whose cell a hit holds.
struct Source {
    // The placement's index above the face's bits.
    placement: u32,

    // The voxel id of the tile's low cell, then `Place::strides`.
    voxel: vec3<u32>,
}

struct Box {
    @builtin(position) clip: vec4<f32>,

    // The point in the tile's cells.
    @location(0) point: vec3<f32>,

    // `Place::ray` from the tile's low corner.
    @location(1) @interpolate(flat) ray: vec3<f32>,

    // The box's low cell in the tile, then the corner past its high cell.
    @location(2) @interpolate(flat) low: vec3<f32>,
    @location(3) @interpolate(flat) high: vec3<f32>,

    // The voxel id of the tile's low cell, then `Place::strides`.
    @location(4) @interpolate(flat) voxel: vec3<u32>,

    // The depth an orthographic ray loses over a unit of its direction.
    @location(5) @interpolate(flat) fall: f32,

    // The axis of the fragment's face, plus 4 on the box's high face.
    @location(6) @interpolate(flat) face: u32,

    // The offset of the tile's bits, then the placement's index above the
    // face's bits.
    @location(7) @interpolate(flat) words: vec2<u32>,

    // The first of the box's entries in `crossers`, then their count.
    @location(8) @interpolate(flat) crossers: vec2<u32>,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> places: array<Place>;
// Per object: its `TileMasks` words.
@group(0) @binding(2) var<storage, read> bits: array<u32>;
// Two words per box: its low cell and its placement's index, then the corner
// past its high cell and the offset of its tile's bits.
@group(0) @binding(3) var<storage, read> boxes: array<vec4<u32>>;
// Two words per box that shares space with a box: the low corner of a box
// around the shared space and the other box's index in `boxes` as bits,
// plus `SOLID` for a solid one, then the high corner.
@group(0) @binding(4) var<storage, read> crossers: array<vec4<f32>>;

// The box's vertex at the grid's `grid`, on the face of code `face`. `low`
// holds the box's low cell and its placement's index. `high` holds the
// corner past the box's high cell and the offset of its tile's bits. `more`
// holds the first of the box's entries in `crossers` and their count.
fn vertex_of(grid: vec3<u32>, face: u32, low: vec4<u32>, high: vec4<u32>, more: vec4<u32>) -> Box {
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let placed = places[low.w];
    let strides = placed.strides.xy;
    let clip = placed.clip * vec4<f32>(vec3<f32>(grid), 1.0);
    let ray = placed.ray.xyz - vec3<f32>(tile) * RAY_W;

    // A corner whose position or ray is not finite goes to the origin,
    // where its box draws nothing.
    return Box(
        select(vec4<f32>(0.0), clip, sound(clip, vec4<f32>(ray, 0.0))),
        vec3<f32>(grid - tile),
        ray,
        vec3<f32>(low.xyz - tile),
        vec3<f32>(high.xyz - tile),
        vec3<u32>(tile.x * strides.x + tile.y * strides.y + tile.z, strides),
        (placed.clip * vec4<f32>(placed.ray.xyz, 0.0)).z,
        face,
        vec2<u32>(high.w, low.w << FACE_BITS),
        more.xy,
    );
}

// A corner of the box, for its back faces. Bit `a` of `corner` picks the
// box's high face on axis `a`.
@vertex
fn place(
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    return vertex_of(corner_of(corner, low, high), corner_face(corner), low, high, more);
}

// `place` for the back faces `inside` shades, under a perspective view.
@vertex
fn place_near(
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    var box = vertex_of(corner_of(corner, low, high), corner_face(corner), low, high, more);

    box.clip.z *= NEARER;

    return box;
}

// A vertex of one of the box's three faces toward the view. `vertex` holds
// the face's axis above its two bits for the face's four vertices.
@vertex
fn place_front(
    @location(0) vertex: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    let front = front_of(vertex, places[low.w].ray.xyz, low.xyz, high.xyz);

    return vertex_of(front.grid, front.face, low, high, more);
}

fn source_of(box: Box) -> Source {
    return Source(box.words.y, box.voxel);
}

// The box at `index` of `boxes`, as a perspective view's ray along
// `direction` through `box` meets it.
fn other_of(box: Box, direction: vec3<f32>, index: u32) -> Other {
    let low = boxes[index * 2u];
    let high = boxes[index * 2u + 1u];
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let own = places[box.words.y >> FACE_BITS];
    let its = places[low.w];
    let strides = its.strides.xy;

    // The direction in meters, then in the other grid's cells.
    let world = own.x.xyz * (direction.x * own.x.w)
        + own.y.xyz * (direction.y * own.y.w)
        + own.z.xyz * (direction.z * own.z.w);

    return Other(
        high.w,
        vec3<f32>(low.xyz - tile),
        vec3<f32>(high.xyz - tile),
        its.ray.xyz - vec3<f32>(tile),
        vec3<f32>(dot(its.x.xyz, world) / its.x.w, dot(its.y.xyz, world) / its.y.w, dot(its.z.xyz, world) / its.z.w),
        Source(low.w << FACE_BITS, vec3<u32>(tile.x * strides.x + tile.y * strides.y + tile.z, strides)),
    );
}

// The packed hit of `cell` in `source`'s tile, across the face on the one
// axis of `across`. The face's normal points up its axis where the ray moves
// down the axis.
fn surface(source: Source, cell: vec3<i32>, across: vec3<bool>, ahead: vec3<bool>) -> vec4<f32> {
    let axis = select(select(0u, 1u, across.y), 2u, across.z);
    let face = axis * 2u + select(1u, 0u, any(across & ahead));
    let voxel = source.voxel.x + u32(cell.x) * source.voxel.y + u32(cell.y) * source.voxel.z + u32(cell.z);

    return unpack4x8unorm(voxel | ((source.placement | face) << frame.voxel_bits));
}
