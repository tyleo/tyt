// A scene's lit corners under one directional light, a word a surface cell
// as `LitCorners` lays them out, over `grid_walk.wgsl` and a walk. An
// invocation fills one surface cell: for each of its faces toward the
// light with no live cell before it, it casts the contract's ray from each
// corner through the placements that can shadow its tile.

// A shadow ray starts this far in from its face's edges.
const INSET: i32 = 8;

struct Fill {
    // The scene's surface cells, then the first of them a dispatch fills.
    cells: u32,
    first: u32,
}

@group(0) @binding(0) var<uniform> fill: Fill;
@group(0) @binding(1) var<storage, read> placements: array<Placement>;
// Per object: its dense cells.
@group(0) @binding(2) var<storage, read> cells: array<u32>;
// Per placement: the direction to the light in its grid.
@group(0) @binding(3) var<storage, read> lights: array<vec4<i32>>;
// Two words per placed tile: its low corner and its placement, then the
// first of its entries in `casters` and their count.
@group(0) @binding(4) var<storage, read> tiles: array<vec4<u32>>;
// The placements that can shadow each placed tile.
@group(0) @binding(5) var<storage, read> casters: array<u32>;
// Per surface cell: its placed tile, above the 12 bits of its index in the
// tile.
@group(0) @binding(6) var<storage, read> surface: array<u32>;
// Per surface cell: its word of lit corners.
@group(0) @binding(7) var<storage, read_write> lit: array<u32>;

// Whether a live cell sits at `position` of `placement`'s grid.
fn live_at(placement: Placement, position: vec3<i32>) -> bool {
    let inside = all(position >= vec3<i32>(0)) && all(position < placement.bounds.xyz);

    return inside && material_at(placement, position) != 0u;
}

// Whether the ray from `origin` in the grid of placement `source` toward
// the light meets no surface in the `count` placements from `first` of
// `casters`.
fn sees_light(origin: vec3<i32>, source: u32, first: u32, count: u32) -> bool {
    for (var n = 0u; n < count; n++) {
        let p = casters[first + n];
        let placement = placements[p];

        var start = origin;

        if p != source {
            start = carry(origin, placements[source], placement);
        }

        let hit = walk(placement, Ray(start, lights[p].xyz, vec3<i32>(0), false));

        if hit.found || hit.starts_inside {
            return false;
        }
    }

    return true;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let at = fill.first + id.x;

    if at >= fill.cells {
        return;
    }

    let entry = surface[at];
    let tile = tiles[(entry >> 12u) * 2u];
    let shadowers = tiles[(entry >> 12u) * 2u + 1u];
    let source = tile.w;
    let placement = placements[source];
    let toward = lights[source].xyz;

    // A cell's index in its tile takes y first.
    let cell = vec3<i32>(tile.xyz)
        + vec3<i32>(vec3<u32>((entry >> 4u) & 15u, (entry >> 8u) & 15u, entry & 15u));

    var word = 0u;

    for (var d = 0; d < 3; d++) {
        let along = pick(toward, d);

        if along == 0 {
            continue;
        }

        // The face on the axis that looks toward the light.
        let up = along > 0;
        var front = cell;

        front[d] += select(-1, 1, up);

        if live_at(placement, front) {
            continue;
        }

        let u = (d + 1) % 3;
        let v = (d + 2) % 3;

        for (var c = 0u; c < 4u; c++) {
            var origin = cell * ONE;

            origin[d] += select(0, ONE, up);
            origin[u] += select(INSET, ONE - INSET, (c & 1u) != 0u);
            origin[v] += select(INSET, ONE - INSET, (c & 2u) != 0u);

            if sees_light(origin, source, shadowers.x, shadowers.y) {
                word |= 1u << (4u * (2u * u32(d) + u32(up)) + c);
            }
        }
    }

    lit[at] = word;
}
