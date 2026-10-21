// The integer walk over each object's `Tree64`. It crosses a region of one
// value in a single jump to the first cell past it, landing on the cell and
// the error terms the walk would reach one step at a time.
// `plain_tree_walk.wgsl` or `counted_tree_walk.wgsl` defines `walk` over it.

const TILE: u32 = 16u;
const TAG_SHIFT: u32 = 29u;
const PAYLOAD: u32 = 0x1fffffffu;
const UNIFORM: u32 = 0u;
const LEAF: u32 = 2u;

// The loop iterations of every walk in this invocation so far.
var<private> iterations: u32;

// A cell's value, and the side of the aligned cube around it that holds only
// that value.
struct Region {
    value: u32,
    size: i32,
}

// The cell where a mirrored walk leaves the box of cells below `high` that
// holds `cell`, and the axis it steps along to reach it.
struct Exit {
    cell: vec3<i32>,
    axis: i32,
}

fn region_at(placement: Placement, position: vec3<i32>) -> Region {
    let base = placement.cells.x;
    let at = vec3<u32>(position);
    let tiles = (vec3<u32>(placement.bounds.xyz) + vec3<u32>(TILE - 1u)) / TILE;
    let tile = at / TILE;
    let slot = cells[base + (tile.x * tiles.y + tile.y) * tiles.z + tile.z];

    if (slot >> TAG_SHIFT) == UNIFORM {
        return Region(slot & PAYLOAD, 16);
    }

    let node = base + (slot & PAYLOAD);
    let child = at % TILE / 4u;
    let bit = child.x * 16u + child.y * 4u + child.z;

    // The mask word holding `bit`, and the children set in the words before
    // it.
    var mask = cells[node];
    var before = 0u;

    if bit >= 32u {
        mask = cells[node + 1u];
        before = countOneBits(cells[node]);
    }

    let shift = bit % 32u;

    if ((mask >> shift) & 1u) == 0u {
        return Region(0u, 4);
    }

    let rank = before + countOneBits(mask & ((1u << shift) - 1u));
    let child_slot = cells[node + 2u + rank];
    let tag = child_slot >> TAG_SHIFT;

    if tag == UNIFORM {
        return Region(child_slot & PAYLOAD, 4);
    }

    let leaf = base + (child_slot & PAYLOAD);
    let bits = 1u << (tag - LEAF);
    let cell = at % 4u;
    let start = (cell.x * 16u + cell.y * 4u + cell.z) * bits;
    let index = (cells[leaf + start / 32u] >> (start % 32u)) & ((1u << bits) - 1u);

    return Region(cells[leaf + 2u * bits + index], 1);
}

fn leave(origin: vec3<i32>, direction: vec3<i32>, cell: vec3<i32>, high: vec3<i32>) -> Exit {
    // The f32 guess of the face the ray leaves across first. A near tie can
    // pick a face too late, which the correction below finds.
    var axis = -1;
    var soonest = 0.0;

    for (var a = 0; a < 3; a++) {
        if direction[a] > 0 {
            let t = f32(high[a] * ONE - origin[a]) / f32(direction[a]);

            if axis < 0 || t < soonest {
                axis = a;
                soonest = t;
            }
        }
    }

    var exit = cell;

    for (var attempt = 0; attempt < 3; attempt++) {
        let e = axis;
        let reach = high[e] * ONE - origin[e];
        let span = ONE * direction[e];

        var past = -1;

        exit = cell;
        exit[e] = high[e];

        for (var a = 0; a < 3; a++) {
            if a == e || direction[a] == 0 {
                continue;
            }

            let x = (f32(origin[a]) + f32(reach) / f32(direction[e]) * f32(direction[a]))
                / f32(ONE);

            var c = clamp(i32(floor(x)), cell[a], high[a]);

            // `r` is exact modulo 2^32 and within i32 near the ray. A
            // crossing at the face on a later axis comes after the exit.
            var r = (origin[a] - c * ONE) * direction[e] + reach * direction[a];

            for (var k = 0; k <= WINDOW && (r < 0 || (r == 0 && a > e)); k++) {
                c -= 1;
                r += span;
            }

            for (var k = 0; k <= WINDOW && (r > span || (r == span && a < e)); k++) {
                c += 1;
                r -= span;
            }

            exit[a] = c;

            if c >= high[a] {
                past = a;
            }
        }

        if past < 0 {
            break;
        }

        // The ray had left across `past` before the guessed face.
        axis = past;
    }

    return Exit(exit, axis);
}

// The walk over the box of `extent` cells from `corner` in the placement's
// grid. The cell before a box's entry face, when it is in the grid, decides
// whether the first cell is a hit, as it would on a walk over the whole grid.
fn tree_walk(placement: Placement, ray: Ray, corner: vec3<i32>, extent: vec3<i32>) -> Hit {
    var box = placement;
    box.bounds = vec4<i32>(extent, 0);

    let shifted = Ray(ray.origin - corner * ONE, ray.direction, ray.end - corner, ray.ends);
    let mirrored = enter(box, shifted);

    if !mirrored.found {
        return miss();
    }

    let grid = placement.bounds.xyz;
    let bounds = extent;
    let origin = mirrored.origin;
    let direction = mirrored.direction;
    let flip = mirrored.flip;
    let last = mirrored.last;
    let entry = mirrored.entry;

    var cell = mirrored.cell;

    // The start cell, or the cell before the entry face. A ray entering the
    // box takes one step in.
    let start_cell = select(cell, bounds - vec3<i32>(1) - cell, flip) + corner;
    var region = Region(0u, 1);

    if entry < 0 {
        region = region_at(placement, start_cell);
    } else if all(start_cell >= vec3<i32>(0)) && all(start_cell < grid) {
        region = Region(region_at(placement, start_cell).value, 1);
    }

    var inside = region.value;

    var start = miss();
    start.starts_inside = entry < 0 && inside != 0u;

    if all(direction == vec3<i32>(0)) {
        return start;
    }

    for (var a = 0; a < 3; a++) {
        if a != entry && cell[a] > last[a] {
            return start;
        }
    }

    var error = error_at(cell, origin, direction);

    // A step up each axis changes the error terms by these.
    let change = ONE * direction;
    let x_change = vec3<i32>(change.y, change.z, 0);
    let y_change = vec3<i32>(-change.x, 0, change.z);
    let z_change = vec3<i32>(0, -change.x, -change.y);

    let meters = length(placement.scale.xyz * vec3<f32>(ray.direction)) / f32(ONE);
    let steps = bounds.x + bounds.y + bounds.z + 3;

    for (var n = 0; n < steps; n++) {
        iterations += 1u;

        var axis = 0;

        if region.size == 1 {
            let first = select(0, 1, error.x > 0);

            axis = select(first, 2, select(error.y, error.z, first == 1) > 0);
            cell += select(vec3<i32>(0), vec3<i32>(1), axis_mask(axis));
            error += select(select(x_change, y_change, axis == 1), z_change, axis == 2);
        } else {
            // The region's cube in mirrored cells, cut to the box. The cube
            // aligns in the grid.
            let size = vec3<i32>(region.size);
            let position = select(cell, bounds - vec3<i32>(1) - cell, flip);
            let at = position + corner;
            let low = at - at % size - corner;
            let high = min(select(low + size, bounds - low, flip), bounds);
            let exit = leave(origin, direction, cell, high);

            cell = exit.cell;
            axis = exit.axis;
            error = error_at(cell, origin, direction);
        }

        if any(cell > last) {
            return start;
        }

        let position = select(cell, bounds - vec3<i32>(1) - cell, flip) + corner;

        region = region_at(placement, position);

        if region.value != 0u && region.value != inside {
            // An entry faces the ray: the cell's upper face on a mirrored
            // axis.
            let ahead = pick(cell, axis) * ONE - pick(origin, axis);
            let distance = f32(ahead) / f32(pick(direction, axis)) * meters;
            let upper = pick(ray.direction, axis) < 0;

            return Hit(true, start.starts_inside, distance, position, axis, upper);
        }

        inside = region.value;
    }

    return start;
}
