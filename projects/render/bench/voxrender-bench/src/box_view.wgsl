// A shaded view of the boxes around each tile's live cells, over
// `mask_march.wgsl`. Multiview passes draw both eyes. A hit shades as the
// contract's opaque surface under one sun and the hemispheres: corner
// occlusion from the tile's words, the sun's corner shadows from
// `LitCorners`, Lambert and GGX, and emission, under the contract's
// tonemap. A smooth face reflects the sky alone. `BoxView` prepends
// `SURFACE` and `MATERIALS`, the offsets in a tile's words of its surface
// cells and its materials.

// Every eye is a perspective view.
const RAY_W: f32 = 1.0;

struct Frame {
    // The unit direction to the sun in the world.
    sun: vec4<f32>,

    // The sun's color times its strength.
    sunlight: vec4<f32>,

    // The hemisphere's light on a face that looks up, then on one that looks
    // down.
    sky: vec4<f32>,
    ground: vec4<f32>,

    // The scene's placements. Each eye has a `Place` for each, the first
    // eye's first.
    placements: u32,
}

struct Place {
    // The placement's grid-to-clip matrix for the eye.
    clip: mat4x4<f32>,

    // The eye in the grid, with w = 1.
    ray: vec4<f32>,

    // The world direction of each of the grid's axes, then the meters a
    // cell spans along it.
    axes: array<vec4<f32>, 3>,
}

// Whose cell a hit holds.
struct Source {
    // The offset of the tile's words.
    tile: u32,

    // The index of the eye's `Place`.
    place: u32,

    // The bits of a cell's entry among the tile's materials, and above
    // them the tile's first material in `materials`.
    header: u32,

    // The first of the tile's words in `lit`.
    lit: u32,

    // The world direction from the eye to the fragment's point, at any
    // length.
    toward: vec3<f32>,
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

    // Zero, since a perspective ray's depth falls by its distance alone.
    @location(4) @interpolate(flat) fall: f32,

    // The axis of the fragment's face, plus 4 on the box's high face.
    @location(5) @interpolate(flat) face: u32,

    // The offset of the tile's words, then the index of the eye's `Place`.
    @location(6) @interpolate(flat) words: vec2<u32>,

    // The first of the box's entries in `crossers`, their count, and the
    // index of the eye's first `Place`.
    @location(7) @interpolate(flat) crossers: vec3<u32>,

    // The tile's materials' word, as `Source::header` holds it, then the
    // first of its words in `lit`. A fragment reads both here, where a
    // read from the tile's words would hold up the reads that follow from
    // it.
    @location(8) @interpolate(flat) shading: vec2<u32>,

    // The world direction from the eye to the point, at any length. A
    // fragment that takes it here reads one of its placement's axes, where
    // one that turns its ray to the world reads all three.
    @location(9) toward: vec3<f32>,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<storage, read> places: array<Place>;
// Per object: its `TileMasks` words, with materials.
@group(0) @binding(2) var<storage, read> bits: array<u32>;
// Three words per material of each tile's materials: its base color and
// metalness, its emission and roughness, and its reflectance at normal
// incidence and occlusion strength.
@group(0) @binding(3) var<storage, read> materials: array<vec4<f32>>;
// Three words per box: its low cell and its placement's index, the corner
// past its high cell and the offset of its tile's words, and in the last
// two parts the tile's materials' word and the first of its words in
// `lit`.
@group(0) @binding(4) var<storage, read> boxes: array<vec4<u32>>;
// Two words per box that shares space with a box: the low corner of a box
// around the shared space and the other box's index in `boxes` as bits,
// plus `SOLID` for a solid one, then the high corner.
@group(0) @binding(5) var<storage, read> crossers: array<vec4<f32>>;
// Each placement's `LitCorners` words.
@group(0) @binding(6) var<storage, read> lit: array<u32>;

// The box's vertex at the grid's `grid` for `eye`, on the face of code
// `face`. `low` holds the box's low cell and its placement's index. `high`
// holds the corner past the box's high cell and the offset of its tile's
// words. `more` holds the first of the box's entries in `crossers`, their
// count, the tile's materials' word, and the first of its words in `lit`.
fn vertex_of(
    eye: u32,
    grid: vec3<u32>,
    face: u32,
    low: vec4<u32>,
    high: vec4<u32>,
    more: vec4<u32>,
) -> Box {
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let index = eye * frame.placements + low.w;
    let clip = places[index].clip * vec4<f32>(vec3<f32>(grid), 1.0);
    let ray = places[index].ray.xyz - vec3<f32>(tile);

    // The eye's ray to the corner, in cells and then in the world.
    let cells = vec3<f32>(grid) - places[index].ray.xyz;
    let axes = places[index].axes;
    let toward = axes[0].xyz * (cells.x * axes[0].w)
        + axes[1].xyz * (cells.y * axes[1].w)
        + axes[2].xyz * (cells.z * axes[2].w);

    // A corner whose position or ray is not finite goes to the origin,
    // where its box draws nothing.
    return Box(
        select(vec4<f32>(0.0), clip, sound(clip, vec4<f32>(ray, 0.0))),
        vec3<f32>(grid - tile),
        ray,
        vec3<f32>(low.xyz - tile),
        vec3<f32>(high.xyz - tile),
        0.0,
        face,
        vec2<u32>(high.w, index),
        vec3<u32>(more.xy, eye * frame.placements),
        more.zw,
        toward,
    );
}

// A corner of the box, for its back faces. Bit `a` of `corner` picks the
// box's high face on axis `a`.
@vertex
fn place(
    @builtin(view_index) eye: u32,
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    return vertex_of(eye, corner_of(corner, low, high), corner_face(corner), low, high, more);
}

// `place` for the back faces `inside` shades.
@vertex
fn place_near(
    @builtin(view_index) eye: u32,
    @location(0) corner: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    var box = vertex_of(eye, corner_of(corner, low, high), corner_face(corner), low, high, more);

    box.clip.z *= NEARER;

    return box;
}

// A vertex of one of the box's three faces toward the eye. `vertex` holds
// the face's axis above its two bits for the face's four vertices.
@vertex
fn place_front(
    @builtin(view_index) eye: u32,
    @location(0) vertex: u32,
    @location(1) low: vec4<u32>,
    @location(2) high: vec4<u32>,
    @location(3) more: vec4<u32>,
) -> Box {
    let ray = places[eye * frame.placements + low.w].ray.xyz;
    let front = front_of(vertex, ray, low.xyz, high.xyz);

    return vertex_of(eye, front.grid, front.face, low, high, more);
}

// The contract's tonemap, Khronos PBR Neutral.
fn tonemap(color: vec3<f32>) -> vec3<f32> {
    let darkest = min(color.r, min(color.g, color.b));
    let offset = select(0.04, darkest - 6.25 * darkest * darkest, darkest < 0.08);
    let lifted = color - offset;
    let peak = max(lifted.r, max(lifted.g, lifted.b));

    if peak < 0.76 {
        return lifted;
    }

    let compressed = 1.0 - 0.0576 / (peak - 0.52);
    let toward_white = 1.0 - 1.0 / (0.15 * (peak - compressed) + 1.0);

    return mix(lifted * (compressed / peak), vec3<f32>(compressed), toward_white);
}

fn source_of(box: Box) -> Source {
    return Source(box.words.x, box.words.y, box.shading.x, box.shading.y, box.toward);
}

// The box at `index` of `boxes`, as the eye's ray along `direction` through
// `box` meets it.
fn other_of(box: Box, direction: vec3<f32>, index: u32) -> Other {
    let low = boxes[index * 3u];
    let high = boxes[index * 3u + 1u];
    let more = boxes[index * 3u + 2u];
    let tile = low.xyz & vec3<u32>(0xfffffff0u);
    let place = box.crossers.z + low.w;
    let own = places[box.words.y];
    let its = places[place];

    // The direction in meters, then in the other grid's cells.
    let world = own.axes[0].xyz * (direction.x * own.axes[0].w)
        + own.axes[1].xyz * (direction.y * own.axes[1].w)
        + own.axes[2].xyz * (direction.z * own.axes[2].w);

    return Other(
        high.w,
        vec3<f32>(low.xyz - tile),
        vec3<f32>(high.xyz - tile),
        its.ray.xyz - vec3<f32>(tile),
        vec3<f32>(dot(its.axes[0].xyz, world) / its.axes[0].w, dot(its.axes[1].xyz, world) / its.axes[1].w, dot(its.axes[2].xyz, world) / its.axes[2].w),
        Source(high.w, place, more.z, more.w, box.toward),
    );
}

// A face's four values of `bits` bits each, from the low bits of `packed`
// up, as a part of their greatest.
fn corners_of(packed: u32, bits: u32) -> vec4<f32> {
    let greatest = (1u << bits) - 1u;

    return vec4<f32>((vec4<u32>(packed) >> (vec4<u32>(0u, 1u, 2u, 3u) * bits)) & vec4<u32>(greatest))
        / f32(greatest);
}

// The value at a point of a face from its corners' `values`. The point
// lies `along.x` of the way across the face on the axis after the face's,
// and `along.y` on the axis after that.
fn across_face(values: vec4<f32>, along: vec2<f32>) -> f32 {
    return mix(mix(values.x, values.y, along.x), mix(values.z, values.w, along.x), along.y);
}

// The hemispheres' light from the unit `direction`, with the sky meeting
// the ground over a band `width` wide in the direction's +Y.
fn sky_light(direction: vec3<f32>, width: f32) -> vec3<f32> {
    return mix(frame.ground.rgb, frame.sky.rgb, clamp(0.5 + direction.y / (2.0 * width), 0.0, 1.0));
}

// The shaded color of `cell` in `source`'s tile, across the face on the one
// axis of `across`. The face's normal points up its axis where the ray moves
// down the axis. The hit lies at `at` in the tile's cells, along a ray of
// `direction` in them.
fn surface(
    source: Source,
    cell: vec3<i32>,
    across: vec3<bool>,
    ahead: vec3<bool>,
    at: vec3<f32>,
    direction: vec3<f32>,
) -> vec4<f32> {
    let tile = source.tile;
    let size = source.header & 15u;
    let index = u32((cell.x * 16 + cell.y) * 16 + cell.z);

    // The cell's material: the tile's first, plus the cell's entry among
    // the tile's.
    var material = source.header >> 4u;

    if size != 0u {
        let entry_at = index * size;

        material += (bits[tile + MATERIALS + 1u + (entry_at >> 5u)] >> (entry_at & 31u))
            & ((1u << size) - 1u);
    }

    let base = materials[material * 3u];
    let glow = materials[material * 3u + 1u];
    let gloss = materials[material * 3u + 2u];
    let up = !any(across & ahead);

    // Where the hit lies across the face, on the axis after the face's and
    // on the one after that.
    let within = at - vec3<f32>(cell);
    let along = clamp(
        vec2<f32>(dot(within, vec3<f32>(across.zxy)), dot(within, vec3<f32>(across.yzx))),
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );

    // The cell's place among its tile's surface cells, and its face.
    let pair = tile + SURFACE + (index >> 5u) * 2u;
    let held = bits[pair];
    let before = bits[pair + 1u];
    let rank = (before & 0xfffu) + countOneBits(held & ((1u << (index & 31u)) - 1u));
    let face = dot(vec3<u32>(across), vec3<u32>(0u, 2u, 4u)) + u32(up);

    // The face's normal points one way along its axis.
    let axis = places[source.place].axes[face >> 1u].xyz;
    let normal = select(-axis, axis, up);

    let open = bits[tile + (before >> 12u) + rank * 2u + (face >> 2u)] >> ((face & 3u) * 8u);
    let occlusion = across_face(corners_of(open, 2u), along);
    let lights = (lit[source.lit + rank] >> (face * 4u)) & 15u;

    // The world direction to the eye.
    // The world direction to the eye.
    let view = -normalize(source.toward);

    let metallic = base.a;
    let roughness = glow.a;
    let alpha = max(roughness * roughness, 1.0e-3);
    let alpha2 = alpha * alpha;
    let reflectance = gloss.rgb;
    let cosine = dot(normal, view);

    var color = glow.rgb;

    // The sun, scaled by the light that reaches the face's corners.
    if lights != 0u {
        let light = across_face(corners_of(lights, 1u), along);

        // The contract's direct term: Lambert, and GGX with Smith
        // visibility and Schlick's Fresnel.
        let toward = frame.sun.xyz;
        let n_dot_l = dot(normal, toward);
        let n_dot_v = max(cosine, 1.0e-4);
        let half = normalize(toward + view);
        let n_dot_h = max(dot(normal, half), 0.0);
        let v_dot_h = max(dot(view, half), 0.0);
        let fresnel = reflectance + (1.0 - reflectance) * pow(1.0 - v_dot_h, 5.0);
        let spread = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;
        let distribution = alpha2 / (3.14159265 * spread * spread);
        let visibility = 0.5
            / (n_dot_l * sqrt(n_dot_v * n_dot_v * (1.0 - alpha2) + alpha2)
                + n_dot_v * sqrt(n_dot_l * n_dot_l * (1.0 - alpha2) + alpha2));
        let diffuse = (1.0 - fresnel) * base.rgb * ((1.0 - metallic) / 3.14159265);

        color += (diffuse + fresnel * (distribution * visibility))
            * (max(n_dot_l, 0.0) * light)
            * frame.sunlight.rgb;
    }

    // The hemispheres, on the face and off it toward the eye.
    let unblocked = 1.0 - gloss.a * (1.0 - occlusion);
    let mirror = normal * (2.0 * cosine) - view;

    color += sky_light(normal, 1.0) * base.rgb * ((1.0 - metallic) * unblocked);
    color += sky_light(mirror, alpha) * reflectance * unblocked;

    return vec4<f32>(tonemap(color), 1.0);
}
