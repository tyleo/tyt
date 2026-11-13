// A scene's lit corners under one directional light, a word a record cell
// as `LitCorners` lays them out, over `grid_walk.wgsl`. An invocation
// fills one record cell: for each of its faces toward the light with no
// opaque cell before it, it casts the contract's ray from each corner
// through the placements that can shadow its tile. A corner is lit where
// some of the light reaches it. What the materials on the way pass of the
// light in each channel lands in the cell's tint as the mean over its lit
// corners.
//
// A cell of a material that passes light fills as a glass cell. Its word
// takes the mean of the light on the corners of its surfaces: its faces
// along the light's axes onto a cell of another material that passes
// light, or onto none. A ray starts on the face and moves toward the
// light, so it tells what falls on the face from the light's side.

// A shadow ray starts this far in from its face's edges.
const INSET: i32 = 8;

// The bits of a word that hold its lit corners. The fill keeps the rest.
const LIT: u32 = 0xffffffu;

struct Fill {
    // The scene's record cells, then the first of them a dispatch fills.
    cells: u32,
    first: u32,

    // How far past a cell's word in `lit` its tint lies, or zero for a
    // scene whose materials pass no light, which keeps no tints.
    tints: u32,
}

struct Light {
    // The direction to the light in a placement's grid.
    toward: vec4<i32>,

    // For a face on each of the grid's axes: one minus the cosine between
    // its normal and the light, to the fifth power.
    fades: vec4<f32>,
}

@group(0) @binding(0) var<uniform> fill: Fill;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per object: its dense cells.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
@group(0) @binding(3) var<storage, read> lights: array<Light>;
// Two words per placed tile: its low corner and its placement, then the
// first of its entries in `casters` and their count.
@group(0) @binding(4) var<storage, read> tiles: array<vec4<u32>>;
// The placements that can shadow each placed tile.
@group(0) @binding(5) var<storage, read> casters: array<u32>;
// Per record cell: its placed tile, above the 12 bits of its index in the
// tile.
@group(0) @binding(6) var<storage, read> recorded: array<u32>;
// Per record cell: its word of lit corners, then with `Fill::tints` its
// tint as four 8-bit parts of one.
@group(0) @binding(7) var<storage, read_write> lit: array<u32>;
// Three per material key: what its surface passes at most, under the share
// no voxel covers. Its reflectance at normal incidence, under whether it
// is opaque. What its reflectance rises by toward grazing.
@group(0) @binding(8) var<storage, read> passes: array<vec4<f32>>;

fn opaque(material: u32) -> bool {
    return passes[material * 3u + 1u].w != 0.0;
}

// Whether an opaque cell sits at `position` of `placement`'s grid.
fn opaque_at(placement: Placement, position: vec3<i32>) -> bool {
    let inside = all(position >= vec3<i32>(0)) && all(position < placement.bounds.xyz);

    return inside && opaque(material_at(placement, position));
}

// The contract's pass of `material`: the share of the light in each channel
// that continues past its surface across a face of `fade`.
fn pass_of(material: u32, fade: f32) -> vec3<f32> {
    let most = passes[material * 3u];
    let reflectance = passes[material * 3u + 1u].rgb + passes[material * 3u + 2u].rgb * fade;

    return most.a + most.rgb * (1.0 - reflectance);
}

// What one placement's grid passes of the light along `ray` in each channel.
// `ray` has no end. The walk runs voxrender's integer walk in i32 alone
// through each surface a shadow ray pays. `fades` holds the fade of a face on
// each axis.
//
// The ray pays the material it starts in at normal incidence and leaves
// it with no exit. It then pays each material it enters, and each it
// leaves for an empty cell, the grid's edge, or a material that passes
// light. An opaque cell passes nothing.
fn passed(placement: Placement, ray: Ray, fades: vec3<f32>) -> vec3<f32> {
    let mirrored = enter(placement, ray);

    if !mirrored.found {
        return vec3<f32>(1.0);
    }

    let bounds = placement.bounds.xyz;
    let origin = mirrored.origin;
    let direction = mirrored.direction;
    let flip = mirrored.flip;
    let last = mirrored.last;
    let entry = mirrored.entry;

    var cell = mirrored.cell;
    var through = vec3<f32>(1.0);

    // The material of the cell the ray is in, and whether the ray is yet
    // to leave the material it starts in.
    var inside = 0u;
    var started = false;

    if entry < 0 {
        inside = material_at(placement, select(cell, bounds - vec3<i32>(1) - cell, flip));

        if inside != 0u {
            if opaque(inside) {
                return vec3<f32>(0.0);
            }

            through = pass_of(inside, 0.0);
            started = true;
        }
    }

    if all(direction == vec3<i32>(0)) {
        return through;
    }

    for (var a = 0; a < 3; a++) {
        if a != entry && cell[a] > last[a] {
            return through;
        }
    }

    var error = error_at(cell, origin, direction);

    // A step up each axis changes the error terms by these.
    let change = ONE * direction;
    let x_change = vec3<i32>(change.y, change.z, 0);
    let y_change = vec3<i32>(-change.x, 0, change.z);
    let z_change = vec3<i32>(0, -change.x, -change.y);

    let strides = voxel_strides(placement, flip);
    var voxel = linear_voxel(placement, select(cell, bounds - vec3<i32>(1) - cell, flip));

    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        let first = select(0, 1, error.x > 0);
        let axis = select(first, 2, select(error.y, error.z, first == 1) > 0);
        let fade = select(select(fades.x, fades.y, axis == 1), fades.z, axis == 2);

        cell += select(vec3<i32>(0), vec3<i32>(1), axis_mask(axis));

        // Only the stepped axis can pass its last cell, at the grid's
        // edge.
        if any(cell > last) {
            if inside != 0u && !started {
                through *= pass_of(inside, fade);
            }

            return through;
        }

        error += select(select(x_change, y_change, axis == 1), z_change, axis == 2);
        voxel += pick(strides, axis);

        let material = cells[placement.cells.x + u32(voxel)];

        if material == inside {
            continue;
        }

        if material != 0u && opaque(material) {
            return vec3<f32>(0.0);
        }

        if inside != 0u && !started {
            through *= pass_of(inside, fade);
        }

        if material != 0u {
            through *= pass_of(material, fade);
        }

        inside = material;
        started = false;
    }

    return through;
}

// What reaches `origin` in the grid of placement `source` of the light in
// each channel, through the `count` placements from `first` of `casters`.
fn light_through(origin: vec3<i32>, source: u32, first: u32, count: u32) -> vec3<f32> {
    var through = vec3<f32>(1.0);

    for (var n = 0u; n < count; n++) {
        let p = casters[first + n];
        let placement = placements[p];

        var start = origin;

        if p != source {
            start = carry(origin, placements[source], placement);
        }

        let light = lights[p];

        through *= passed(placement, Ray(start, light.toward.xyz, vec3<i32>(0), false), light.fades.xyz);

        if all(through == vec3<f32>(0.0)) {
            break;
        }
    }

    return through;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let at = fill.first + id.x;

    if at >= fill.cells {
        return;
    }

    let entry = recorded[at];
    let tile = tiles[(entry >> 12u) * 2u];
    let shadowers = tiles[(entry >> 12u) * 2u + 1u];
    let source = tile.w;
    let placement = placements[source];
    let toward = lights[source].toward.xyz;

    // A cell's index in its tile takes y first.
    let cell = vec3<i32>(tile.xyz)
        + vec3<i32>(vec3<u32>((entry >> 4u) & 15u, (entry >> 8u) & 15u, entry & 15u));

    var word = 0u;

    // The light the lit corners take, and their count.
    var light = vec3<f32>(0.0);
    var corners = 0.0;

    let own = material_at(placement, cell);

    if !opaque(own) {
        for (var n = 0; n < 6; n++) {
            let d = n >> 1u;
            let up = (n & 1) == 1;

            if pick(toward, d) == 0 {
                continue;
            }

            var beside = cell;

            beside[d] += select(-1, 1, up);

            var other = 0u;

            if all(beside >= vec3<i32>(0)) && all(beside < placement.bounds.xyz) {
                other = material_at(placement, beside);
            }

            if other == own || opaque(other) {
                continue;
            }

            let u = (d + 1) % 3;
            let v = (d + 2) % 3;

            for (var c = 0u; c < 4u; c++) {
                var origin = cell * ONE;

                origin[d] += select(0, ONE, up);
                origin[u] += select(INSET, ONE - INSET, (c & 1u) != 0u);
                origin[v] += select(INSET, ONE - INSET, (c & 2u) != 0u);

                light += light_through(origin, source, shadowers.x, shadowers.y);
                corners += 1.0;
            }
        }

        lit[at] = pack4x8unorm(vec4<f32>(light / max(corners, 1.0), 0.0));

        return;
    }

    for (var d = 0; d < 3; d++) {
        let along = pick(toward, d);

        if along == 0 {
            continue;
        }

        // The face on the axis that looks toward the light.
        let up = along > 0;
        var front = cell;

        front[d] += select(-1, 1, up);

        if opaque_at(placement, front) {
            continue;
        }

        let u = (d + 1) % 3;
        let v = (d + 2) % 3;

        for (var c = 0u; c < 4u; c++) {
            var origin = cell * ONE;

            origin[d] += select(0, ONE, up);
            origin[u] += select(INSET, ONE - INSET, (c & 1u) != 0u);
            origin[v] += select(INSET, ONE - INSET, (c & 2u) != 0u);

            let through = light_through(origin, source, shadowers.x, shadowers.y);

            if any(through > vec3<f32>(0.0)) {
                word |= 1u << (4u * (2u * u32(d) + u32(up)) + c);
                light += through;
                corners += 1.0;
            }
        }
    }

    lit[at] = (lit[at] & ~LIT) | word;

    if fill.tints != 0u {
        lit[at + fill.tints] = pack4x8unorm(vec4<f32>(light / max(corners, 1.0), 0.0));
    }
}
