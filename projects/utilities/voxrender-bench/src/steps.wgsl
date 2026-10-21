// Each invocation reads `STEPS` cells along a diagonal through the grid, as a
// DDA steps x, then y, then z. Each read's address depends on the last read.
// A read of the miss ends the loop, as a hit ends a walk.

override STEPS: u32 = 8u;

override SIZE: u32 = 256u;

const BRICK: u32 = 4u;

struct Steps {
    width: u32,
    height: u32,
    // Zero, which the compiler cannot know.
    salt: u32,
    // A value no cell holds.
    miss: u32,
}

@group(0) @binding(0) var<uniform> steps: Steps;
@group(0) @binding(1) var<storage, read> cells: array<u32>;
@group(0) @binding(2) var volume: texture_3d<u32>;
@group(0) @binding(3) var layers: texture_2d_array<u32>;
@group(0) @binding(4) var<storage, read> masks: array<u32>;
@group(0) @binding(5) var<storage, read_write> ends: array<u32>;

// The cell of step `n` for the pixel `id`. The image's x runs along the
// grid's z and its y along the grid's y.
fn cell_at(id: vec3<u32>, n: u32, offset: u32) -> vec3<u32> {
    let start = vec3<u32>(offset, id.y * SIZE / steps.height, id.x * SIZE / steps.width);

    return (start + vec3<u32>(n + 2u, n + 1u, n) / 3u) % SIZE;
}

fn inside(id: vec3<u32>) -> bool {
    return id.x < steps.width && id.y < steps.height;
}

@compute @workgroup_size(8, 8)
fn buffer_steps(@builtin(global_invocation_id) id: vec3<u32>) {
    if !inside(id) {
        return;
    }

    var offset = 0u;

    for (var n = 0u; n < STEPS; n++) {
        let cell = cell_at(id, n, offset);
        let value = cells[(cell.x * SIZE + cell.y) * SIZE + cell.z];

        if value == steps.miss {
            ends[0] = n;
            return;
        }

        offset += value & steps.salt;
    }
}

@compute @workgroup_size(8, 8)
fn volume_steps(@builtin(global_invocation_id) id: vec3<u32>) {
    if !inside(id) {
        return;
    }

    var offset = 0u;

    for (var n = 0u; n < STEPS; n++) {
        let cell = cell_at(id, n, offset);
        let value = textureLoad(volume, cell.zyx, 0).x;

        if value == steps.miss {
            ends[0] = n;
            return;
        }

        offset += value & steps.salt;
    }
}

@compute @workgroup_size(8, 8)
fn layer_steps(@builtin(global_invocation_id) id: vec3<u32>) {
    if !inside(id) {
        return;
    }

    var offset = 0u;

    for (var n = 0u; n < STEPS; n++) {
        let cell = cell_at(id, n, offset);
        let value = textureLoad(layers, cell.zy, cell.x, 0).x;

        if value == steps.miss {
            ends[0] = n;
            return;
        }

        offset += value & steps.salt;
    }
}

// Each step reads its brick's mask word and its bit's rank, as the tree walk
// does.
@compute @workgroup_size(8, 8)
fn mask_steps(@builtin(global_invocation_id) id: vec3<u32>) {
    if !inside(id) {
        return;
    }

    let bricks = SIZE / BRICK;

    var offset = 0u;

    for (var n = 0u; n < STEPS; n++) {
        let cell = cell_at(id, n, offset);
        let brick = cell / BRICK;
        let pair = ((brick.x * bricks + brick.y) * bricks + brick.z) * 2u;
        let local = cell % BRICK;
        let bit = local.x * 16u + local.y * 4u + local.z;

        var mask = masks[pair];
        var before = 0u;

        if bit >= 32u {
            mask = masks[pair + 1u];
            before = countOneBits(masks[pair]);
        }

        let shift = bit % 32u;
        let rank = before + countOneBits(mask & ((1u << shift) - 1u));
        let value = select(0u, rank, ((mask >> shift) & 1u) != 0u);

        if value == steps.miss {
            ends[0] = n;
            return;
        }

        offset += value & steps.salt;
    }
}
