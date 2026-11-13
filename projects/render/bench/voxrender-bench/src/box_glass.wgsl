// The contract's transmission over a `BoxView`: a fragment walks its box
// through the surfaces between materials that pass light and writes the
// light they add and the share of the light behind the box that they pass.
// The target's blend scales what it holds by that share and adds the light.
// `box_view.wgsl` and `mask_march.wgsl` come before this file.
//
// A box's words hold:
//
// 1. In `words.x`, the offset of its tile's `GlassTiles` words in `bits`
// 2. In `crossers.x`, the box around the tile's surfaces, 4 bits a side: its
//    low cell in the tile, then its high cell
// 3. In `shading.y`, the first of the tile's words in `lit`
//
// A busy cell's word holds where its material's values start in
// `materials`, which `past` lists, and its place among the tile's words in
// `lit`. The cell's word in `lit` holds the sun's light on the cell's
// surfaces as four 8-bit parts of one.
//
// A box spans its tile's opaque cells too. The opaque view's depth holds
// the faces of the box around those, which can lie before a surface they
// do not hide. A walk that starts on its box then starts no later. An opaque
// cell ends the walk.
//
// A surface reflects the hemispheres alone. A ray that starts inside a
// material leaves it for nothing in the box that holds the eye. A walk in
// any other box starts over and charges that exit.

// `GlassTiles::ROWS`, the offset in a tile's words of its busy cells'
// words, and `GlassTiles::OPAQUE`, an opaque cell's word.
const BUSY_CELLS: u32 = 256u;
const OPAQUE: u32 = 64u;

// A face takes the sun's highlight where pi times its GGX distribution is
// over one part in this.
const LOBE: f32 = 64.0;

struct Glass {
    @location(0) @blend_src(0) light: vec4<f32>,
    @location(0) @blend_src(1) kept: vec4<f32>,
}

// A walk's light so far, and the share of the light behind it that the
// surfaces so far pass.
struct Walked {
    light: vec3<f32>,
    through: vec3<f32>,

    // The surface the ray last entered by: its material's place under its
    // axis, the light it adds under its coverage, and what it passes.
    entered: u32,
    added: vec3<f32>,
    passed: vec3<f32>,
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

// What a fragment's ray sees of a face on each of its grid's axes, a part
// an axis. A surface's normal faces back along the ray, so one ray meets
// every face on an axis from the same side.
struct Sides {
    // The cosines between the normal and the view, the sun, and the half
    // vector of the two.
    cosines: vec3<f32>,
    suns: vec3<f32>,
    halves: vec3<f32>,

    // The world +Y of the normal, then of the view.
    ups: vec3<f32>,
    view_up: f32,

    // One minus the cosine between the view and the half vector, to the
    // fifth power.
    glint: f32,

    // The first of the tile's words in `lit`.
    lit: u32,
}

fn sides_of(box: Box) -> Sides {
    let axes = places[box.words.y].axes;
    let view = -normalize(box.toward);

    // The view, the sun, and their half vector along the grid's axes.
    let seen = vec3<f32>(dot(axes[0].xyz, view), dot(axes[1].xyz, view), dot(axes[2].xyz, view));
    let sun = vec3<f32>(
        dot(axes[0].xyz, frame.sun.xyz),
        dot(axes[1].xyz, frame.sun.xyz),
        dot(axes[2].xyz, frame.sun.xyz),
    );
    let half = normalize(sun + seen);

    // A normal points up its axis where the view does.
    let turn = select(vec3<f32>(-1.0), vec3<f32>(1.0), seen >= vec3<f32>(0.0));
    let off = 1.0 - max(dot(seen, half), 0.0);

    return Sides(
        abs(seen),
        turn * sun,
        max(turn * half, vec3<f32>(0.0)),
        turn * vec3<f32>(axes[0].y, axes[1].y, axes[2].y),
        view.y,
        off * off * off * off * off,
        box.shading.y,
    );
}

// The hemispheres' light from a direction whose +Y is `up`, with the sky
// meeting the ground over a band `band` is half the inverse of.
fn sky_from(up: f32, band: f32) -> vec3<f32> {
    return mix(frame.ground.rgb, frame.sky.rgb, clamp(0.5 + up * band, 0.0, 1.0));
}

// `walked` past the surface of the cell of word `cell` that the ray crosses
// on the one axis of `across`, or `walked` where the cell's face there is
// no surface. With `leaves` the ray crosses out of the cell.
//
// A material's values:
//
// 1. What its surface passes at normal incidence, under its coverage
// 2. What that falls by toward grazing, under half the inverse of its GGX
//    alpha
// 3. Its reflectance at normal incidence in one channel, what that rises
//    by toward grazing, its alpha's square, and what follows: 0 for
//    nothing, 1 for its diffuse color, 2 for the three after too
// 4. Its diffuse color
// 5. Its reflectance at normal incidence, what that rises by toward
//    grazing, and its emission, a color each. A material takes them where
//    its reflectance has a hue or it glows
fn past(walked: Walked, sides: Sides, cell: u32, across: vec3<bool>, ahead: vec3<bool>, leaves: bool) -> Walked {
    let axis = dot(vec3<u32>(across), vec3<u32>(0u, 1u, 2u));
    let up = any(across & ahead);

    if ((cell >> (axis * 2u + u32(up == leaves))) & 1u) == 0u {
        return walked;
    }

    let at = cell >> 20u;

    // A ray that leaves a material by the axis it entered on meets the
    // same surface again. It takes the sun as the cell it entered by did.
    if leaves && walked.entered == (at << 2u | axis) {
        return Walked(
            walked.light + walked.through * walked.added,
            walked.through * walked.passed,
            walked.entered,
            walked.added,
            walked.passed,
        );
    }

    let side = vec3<f32>(across);
    let cosine = dot(sides.cosines, side);
    let rest = 1.0 - cosine;
    let square = rest * rest;
    let fade = square * square * rest;
    let most = materials[at];
    let fall = materials[at + 1u];
    let passed = most.rgb - fall.rgb * fade;
    let through = walked.through * passed;

    let gloss = materials[at + 2u];
    let above = dot(sides.ups, side);
    let n_dot_l = dot(sides.suns, side);

    // The hemispheres along the mirror ray, by the reflectance.
    let mirrored = sky_from(above * (2.0 * cosine) - sides.view_up, fall.a);
    var reflectance = vec3<f32>(gloss.x + gloss.y * fade);
    var fresnel = vec3<f32>(gloss.x + (1.0 - gloss.x) * sides.glint);
    var color = vec3<f32>(0.0);

    if gloss.w > 1.5 {
        let hue = materials[at + 4u].rgb;

        reflectance = hue + materials[at + 5u].rgb * fade;
        fresnel = hue + (1.0 - hue) * sides.glint;
        color = materials[at + 6u].rgb;
    }

    color += mirrored * reflectance;

    // The cell's word in `lit`: the sun's light that reaches its
    // surfaces.
    let shade = sides.lit + ((cell >> 8u) & 0xfffu);

    // The diffuse color under the hemispheres and the sun.
    if gloss.w > 0.5 {
        let diffuse = materials[at + 3u].rgb;

        color += diffuse * sky_from(above, 0.5);

        if n_dot_l > 0.0 {
            color += diffuse
                * (1.0 - fresnel)
                * (n_dot_l / 3.14159265)
                * frame.sunlight.rgb
                * unpack4x8unorm(lit[shade]).rgb;
        }
    }

    // The sun's highlight, as `sun_light` gives it, where the face's lobe
    // holds the half vector.
    let n_dot_h = dot(sides.halves, side);
    let alpha2 = gloss.z;
    let spread = n_dot_h * n_dot_h * (alpha2 - 1.0) + 1.0;

    if n_dot_l > 0.0 && spread * spread < alpha2 * LOBE {
        let n_dot_v = max(cosine, 1.0e-4);
        let distribution = alpha2 / (3.14159265 * spread * spread);
        let visibility = 0.5
            / (n_dot_l * sqrt(n_dot_v * n_dot_v * (1.0 - alpha2) + alpha2)
                + n_dot_v * sqrt(n_dot_l * n_dot_l * (1.0 - alpha2) + alpha2));

        color += fresnel
            * (distribution * visibility * n_dot_l)
            * frame.sunlight.rgb
            * unpack4x8unorm(lit[shade]).rgb;
    }

    let added = most.a * color;

    return Walked(walked.light + walked.through * added, through, at << 2u | axis, added, passed);
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
    let packed = vec3<u32>(box.crossers.x);
    let low = vec3<f32>((packed >> vec3<u32>(0u, 4u, 8u)) & vec3<u32>(15u));
    let high = vec3<f32>((packed >> vec3<u32>(12u, 16u, 20u)) & vec3<u32>(15u)) + 1.0;
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

    let sides = sides_of(box);

    var march = March(bit_of(cell), next, entry, 0.0);
    var walked = Walked(vec3<f32>(0.0), vec3<f32>(1.0), 3u, vec3<f32>(0.0), vec3<f32>(1.0));

    // The word of the cell the ray stands in.
    var word = busy_cell(tile, march.bit);

    if enters && word != OPAQUE {
        walked = past(walked, sides, word, entry, ahead, false);
    }

    while word != OPAQUE && advance(&march, rate, strides, end) {
        let before = word;

        word = busy_cell(tile, march.bit);

        // An opaque cell ends the walk with no exit: its face is the
        // opaque view's.
        if word != OPAQUE {
            walked = past(walked, sides, before, march.across, ahead, true);
            walked = past(walked, sides, word, march.across, ahead, false);
        }
    }

    // The ray leaves the surfaces' box across the axis the last step
    // found.
    if word != OPAQUE {
        walked = past(walked, sides, word, march.across, ahead, true);
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
