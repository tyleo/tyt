// The contract's transmission over a `BoxView`: a fragment walks its box
// through the surfaces between materials that pass light and writes the
// light they add and the share of the light behind the box that they pass.
// The target's blend scales what it holds by that share and adds the light.
// `box_view.wgsl` and `mask_march.wgsl` come before this file.
//
// A box's `words.x` holds the offset of its tile's `GlassTiles` words in
// `bits`, and its `crossers.x` the box around the tile's surfaces, 4 bits a
// side: its low cell in the tile, then its high cell. A busy cell's word
// holds where its material's four values start in `materials`: the three
// an opaque material has, then its coverage and its transmission.
//
// A box spans its tile's opaque cells too. The opaque view's depth holds
// the faces of the box around those, which can lie before a surface they
// do not hide. A walk that starts on its box then starts no later. An opaque
// cell ends the walk.
//
// A surface takes the sun whole, with no shadow, and reflects the
// hemispheres alone. A ray that starts inside a material pays nothing for
// it.

// `GlassTiles::ROWS`, the offset in a tile's words of its busy cells'
// words, and `GlassTiles::OPAQUE`, an opaque cell's word.
const BUSY_CELLS: u32 = 256u;
const OPAQUE: u32 = 64u;

struct Glass {
    @location(0) @blend_src(0) light: vec4<f32>,
    @location(0) @blend_src(1) kept: vec4<f32>,
}

// A walk's light so far, and the share of the light behind it that the
// surfaces so far pass.
struct Walked {
    light: vec3<f32>,
    through: vec3<f32>,
}

// The word of the tile's cell at `bit`, or zero for a cell that is not
// busy: a cell with no surface.
fn busy_cell(tile: u32, bit: i32) -> u32 {
    let cells = cells_at(tile, bit);

    if !live_in(cells, bit) {
        return 0u;
    }

    let rank = (cells >> 16u) + countOneBits(cells & ((1u << (u32(bit) & 15u)) - 1u));

    return bits[tile + BUSY_CELLS + rank];
}

// `walked` past the surface of the cell of word `cell` that the ray crosses
// on the one axis of `across`, or `walked` where the cell's face there is
// no surface. With `leaves` the ray crosses out of the cell. The surface's
// normal faces back along the ray either way.
fn past(walked: Walked, box: Box, cell: u32, across: vec3<bool>, ahead: vec3<bool>, leaves: bool) -> Walked {
    let axis = dot(vec3<u32>(across), vec3<u32>(0u, 1u, 2u));
    let up = any(across & ahead);

    if ((cell >> (axis * 2u + u32(up == leaves))) & 1u) == 0u {
        return walked;
    }

    let at = cell >> 8u;
    let base = materials[at];
    let glow = materials[at + 1u];
    let gloss = materials[at + 2u];
    let cover = materials[at + 3u];

    let side = places[box.words.y].axes[axis];
    let normal = select(side.xyz, -side.xyz, up);
    let view = -normalize(box.toward);
    let cosine = max(dot(normal, view), 0.0);

    let metallic = base.a;
    let roughness = glow.a;
    let alpha = max(roughness * roughness, 1.0e-3);

    // The contract's pass. The share that transmits loses what the surface
    // reflects, which rises toward grazing as far as the roughness lets it.
    let transmitted = cover.y * (1.0 - metallic);
    let grazing = gloss.rgb + (max(vec3<f32>(1.0 - roughness), gloss.rgb) - gloss.rgb) * pow(1.0 - cosine, 5.0);
    let passed = (1.0 - cover.x) + cover.x * transmitted * (1.0 - grazing) * base.rgb;

    let diffuse = base.rgb * ((1.0 - metallic) * (1.0 - cover.y));
    let reflectance = gloss.rgb * (1.0 - transmitted) + grazing * transmitted;
    let mirror = normal * (2.0 * cosine) - view;

    let color = glow.rgb
        + sun_light(normal, view, diffuse, gloss.rgb, alpha * alpha)
        + sky_light(normal, 1.0) * diffuse
        + sky_light(mirror, alpha) * reflectance;

    return Walked(walked.light + walked.through * (cover.x * color), walked.through * passed);
}

// What the ray from `start` along `direction` meets of the box's surfaces,
// to where it leaves the box around them or meets an opaque cell. A ray
// that `enters` starts on the face of the box across the one axis of
// `entry`. Any other ray starts at an eye inside the box.
fn glass_walk(box: Box, start: vec3<f32>, direction: vec3<f32>, entry: vec3<bool>, enters: bool) -> Glass {
    let tile = box.words.x;
    let ahead = direction >= vec3<f32>(0.0);
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    // How far along the ray it enters and leaves the box around the
    // surfaces. A ray that misses that box meets none.
    let sides = vec3<u32>(box.crossers.x);
    let low = vec3<f32>((sides >> vec3<u32>(0u, 4u, 8u)) & vec3<u32>(15u));
    let high = vec3<f32>((sides >> vec3<u32>(12u, 16u, 20u)) & vec3<u32>(15u)) + 1.0;
    let turn = select(-rate, rate, ahead);
    let nears = (select(high, low, ahead) - start) * turn;
    let fars = (select(low, high, ahead) - start) * turn;
    let far = min(fars.x, min(fars.y, fars.z));

    if far <= max(max(nears.x, max(nears.y, nears.z)), 0.0) {
        discard;
    }

    let cell = clamp(floor(start), box.low, box.high - 1.0);
    let strides = select(vec3<i32>(-16, -256, -1), vec3<i32>(16, 256, 1), ahead);
    let within = start - cell;
    let next = abs(select(within, 1.0 - within, ahead)) * rate;
    let end = far * 0.99999;

    var march = March(bit_of(cell), next, entry, 0.0);
    var walked = Walked(vec3<f32>(0.0), vec3<f32>(1.0));

    // The word of the cell the ray stands in.
    var word = busy_cell(tile, march.bit);

    if enters && word != OPAQUE {
        walked = past(walked, box, word, entry, ahead, false);
    }

    while word != OPAQUE && advance(&march, rate, strides, end) {
        let before = word;

        word = busy_cell(tile, march.bit);

        // An opaque cell ends the walk with no exit: its face is the
        // opaque view's.
        if word != OPAQUE {
            walked = past(walked, box, before, march.across, ahead, true);
            walked = past(walked, box, word, march.across, ahead, false);
        }
    }

    // The ray leaves the surfaces' box across the axis the last step
    // found.
    if word != OPAQUE {
        walked = past(walked, box, word, march.across, ahead, true);
    }

    // The target holds tonemapped light, and the blend runs on that. Under
    // its shoulder the tonemap takes 0.04 off a color, which the light
    // behind the glass has already paid. A bright light takes the tonemap
    // itself.
    var light = walked.light;

    if max(light.r, max(light.g, light.b)) > 0.72 {
        light = tonemap(light + 0.04);
    }

    return Glass(
        vec4<f32>(max(light - 0.04 * (1.0 - walked.through), vec3<f32>(0.0)), 0.0),
        vec4<f32>(walked.through, 1.0),
    );
}

// A front face's fragment. The rasterizer's point is where the ray enters
// the box. An eye the box holds draws through `glass_held` in its place.
@fragment
fn glass(box: Box) -> Glass {
    let reach = places[box.words.y].ray.w;

    if all(box.ray >= box.low - reach) && all(box.ray <= box.high + reach) {
        discard;
    }

    return glass_walk(box, box.point, direction_of(box), face_of(box), true);
}

// A back face's fragment of a box that holds a perspective view's eye. The
// ray starts where it enters the box, or at the eye inside the box, one
// direction back.
@fragment
fn glass_held(box: Box) -> Glass {
    let direction = direction_of(box);
    let ahead = direction >= vec3<f32>(0.0);
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));
    let faces = abs(box.point - select(box.high, box.low, ahead)) * rate;
    let entry = min(faces.x, min(faces.y, faces.z));
    let before = min(entry, 1.0);

    return glass_walk(box, box.point - direction * before, direction, least(faces), entry <= 1.0);
}
