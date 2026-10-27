// Each invocation runs a dependent chain of the adds and compares a walk's
// step makes, in i32 or f32.

// Runs the chain in f32 instead.
override FLOAT_CHAIN: bool = false;

const LINKS: u32 = 64u;

struct Chain {
    width: u32,
    height: u32,
    // Each link adds `step`, then takes off `limit` once the sum reaches it.
    step: u32,
    limit: u32,
}

@group(0) @binding(0) var<uniform> chain: Chain;
// Written only by a chain that ends on `limit`, which none does, so the
// compiler keeps every link with no store per invocation.
@group(0) @binding(1) var<storage, read_write> ends: array<u32>;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= chain.width || id.y >= chain.height {
        return;
    }

    let seed = (id.y * chain.width + id.x) % chain.limit;

    var end = 0u;

    if FLOAT_CHAIN {
        let step = f32(chain.step);
        let limit = f32(chain.limit);

        var x = f32(seed);

        for (var n = 0u; n < LINKS; n++) {
            x += step;
            x = select(x, x - limit, x >= limit);
        }

        end = u32(x);
    } else {
        let step = i32(chain.step);
        let limit = i32(chain.limit);

        var x = i32(seed);

        for (var n = 0u; n < LINKS; n++) {
            x += step;
            x = select(x, x - limit, x >= limit);
        }

        end = u32(x);
    }

    if end == chain.limit {
        ends[0] = 1u;
    }
}
