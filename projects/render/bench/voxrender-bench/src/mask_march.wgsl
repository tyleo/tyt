// A march through a tile's bits in a box raster's fragments. A fragment
// steps its ray a cell at a time, from the point the rasterizer gives it on
// its box.
//
// `front`, `crossed`, `inside`, and `held` discard a miss and keep the depth
// their vertices give, which lets the depth test run before the march.
// `exact` writes the hit's depth. `solid` marches nothing.
//
// Under face depth the nearest face wins a pixel. That face must then hold
// the pixel's first hit, so a front face's fragment finds the first hit of
// any placement inside its box. `front` serves a box that shares space with
// no other placement's box. `crossed` steps through its tile and then
// through each box of another placement that shares space with its box. A
// hit past its box is some other box's to find.
//
// `inside` shades the back faces of a box a ray can start inside. A hit in
// such a box lies before the box's back face. The front face of the box
// behind can share that face's plane. A vertex entry for `inside` scales its
// depth by `NEARER`, which keeps the hit ahead of that box's.
//
// `held` shades the back faces of a box that holds the view's eye and
// shares space with other placements' boxes. Its ray starts at the eye, so
// the first hit it finds in its tile and in those boxes is the pixel's. A
// vertex entry for `held` gives it the nearest depth. Its box also draws its
// front faces through `crossed` for an eye outside it.
//
// The WGSL that includes this file defines:
//
// 1. `RAY_W`, 1 under a perspective view and 0 under an orthographic one
// 2. `Box`, the fragment's inputs: `clip`, its position. `point`, its point
//    in the tile's cells. `ray`, a perspective view's eye from the tile's low
//    corner, or the reverse of an orthographic view's direction. `low`, the
//    box's low cell in the tile. `high`, the corner past the box's high
//    cell. `words`, whose x holds the offset of the tile's words in `bits`.
//    `fall`, the depth an orthographic ray loses over a unit of its
//    direction. `face`, the axis of the fragment's face, plus 4 on the box's
//    high face. `crossers`, the first of the box's entries in `crossers` and
//    their count
// 3. `Cells`, what one load gives of a tile's words as `TileMasks` lays
//    them out. `cells_at` loads the `Cells` that hold a cell's bit, and
//    `live_in` tells whether the cell is live in them
// 4. `crossers`, whose entries each hold a box of another placement that
//    shares space with a box, as `BoxCrosser` lays it out
// 5. `Source`, what tells `surface` whose cell it shades. `source_of` gives
//    the source of the fragment's box
// 6. `other_of`, the `Other` of a `crossers` entry's box: the ray through
//    the fragment as that box's tile sees it
// 7. `surface`, what a fragment writes for a hit cell of a source, given the
//    `Cells` that hold the cell's bit, the one axis of the face the ray
//    crossed into the cell, the axes the ray moves up, the hit's point in
//    the tile's cells, and the ray's direction in them
//
// Its vertex entries place a corner only when `sound` holds for the corner's
// position and its `ray`. They take a box's back faces from `BoxDraw`'s
// corners through `corner_of` and `corner_face`, and its faces toward the
// view from `BoxDraw`'s front corners through `front_of`.

// What a perspective view's reversed depth scales by to come a part in 1024
// nearer.
const NEARER: f32 = 1.0 + 1.0 / 1024.0;

struct Fragment {
    @location(0) hit: vec4<f32>,
    @builtin(frag_depth) depth: f32,
}

// The bit of a `crossers` entry that marks a solid box.
const SOLID: u32 = 0x80000000u;

struct Hit {
    found: bool,

    // The hit cell's bit in its tile.
    bit: i32,

    // The one axis of the face the ray crossed into the cell.
    across: vec3<bool>,

    // How far along the ray the hit lies. A miss holds how far the ray's
    // box ends.
    along: f32,

    // The `Cells` that hold the hit cell's bit. A hit hands them on, where a
    // second load would hold up the reads that follow from it.
    cells: Cells,
}

// A box of another placement, as a fragment whose box shares space with it
// reads it.
struct Other {
    // The offset of the box's tile's words in `bits`.
    tile: u32,

    // The box's low cell in its tile, then the corner past its high cell.
    low: vec3<f32>,
    high: vec3<f32>,

    // The view's eye from the tile's low corner, and the ray's direction,
    // both in the tile's cells. The fragment's point lies one direction
    // from the eye.
    eye: vec3<f32>,
    direction: vec3<f32>,

    source: Source,
}

// Whether each component of `a` and of `b` is a finite number. The test
// reads bits, which a compiler that takes every number as finite leaves
// alone.
fn sound(a: vec4<f32>, b: vec4<f32>) -> bool {
    let top = vec4<u32>(0x7f800000u);

    return all((bitcast<vec4<u32>>(a) & top) != top) && all((bitcast<vec4<u32>>(b) & top) != top);
}

// The grid point of a box's corner. Bit `a` of `corner` picks the box's high
// face on axis `a`.
fn corner_of(corner: u32, low: vec4<u32>, high: vec4<u32>) -> vec3<u32> {
    let far = ((vec3<u32>(corner) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) == vec3<u32>(1u);

    return select(low.xyz, high.xyz, far);
}

// The code of the face whose triangles start at `corner`: its axis, plus 4
// for the box's high face. `BoxDraw` starts the low face on axis `a` at
// corner `a` and the high face at corner `7 - a`.
fn corner_face(corner: u32) -> u32 {
    return min(corner, 7u - corner) | (corner & 4u);
}

// A vertex of a face toward the view.
struct Front {
    // The vertex's grid point.
    grid: vec3<u32>,

    // The face's code: its axis, plus 4 for the box's high face.
    face: u32,
}

// The vertex `vertex` of the box from `low` to `high`, one of the 7 corners
// of its three faces toward the view. Bit `b` of `vertex` picks the far
// side on the axis the bit stands for, and no vertex sets all three. `ray`
// is the placement's `Place::ray`.
//
// A perspective view's eye past the box's middle sees the high face, as
// does an orthographic view whose reverse direction points up the axis. An
// eye between the two faces sees neither, and the face then winds away from
// it.
//
// Bit 0 stands for x. Bits 1 and 2 stand for y and z, or for z and y where
// the view sees an odd count of high faces, which keeps `BoxDraw`'s
// triangles wound counterclockwise from outside. A face's triangles start
// at its corner that no other face holds. That corner has a single clear
// bit, which gives the face.
fn front_of(vertex: u32, ray: vec3<f32>, low: vec3<u32>, high: vec3<u32>) -> Front {
    let toward = ray * 2.0 > vec3<f32>(low + high) * RAY_W;
    let odd = (toward.x != toward.y) != toward.z;
    let bits = ((vec3<u32>(vertex) >> vec3<u32>(0u, 1u, 2u)) & vec3<u32>(1u)) == vec3<u32>(1u);
    let far = vec3<bool>(bits.x, select(bits.y, bits.z, odd), select(bits.z, bits.y, odd));

    // The clear bit of a corner one face holds, then the axis it stands for.
    let clear = select(select(2u, 1u, !bits.y), 0u, !bits.x);
    let axis = select(clear, 3u - clear, odd && clear != 0u);

    return Front(
        select(select(low, high, toward), select(high, low, toward), far),
        axis | (u32(toward[axis]) << 2u),
    );
}

// The one axis of the least component of `v`. A tie goes to the earlier
// axis.
fn least(v: vec3<f32>) -> vec3<bool> {
    let x = v.x <= v.y && v.x <= v.z;
    let y = !x && v.y <= v.z;

    return vec3<bool>(x, y, !x && !y);
}

// The cell's bit in its tile.
fn bit_of(cell: vec3<f32>) -> i32 {
    return i32(dot(cell, vec3<f32>(16.0, 256.0, 1.0)));
}

// A ray's place among a tile's cells.
struct March {
    // The cell's bit in the tile.
    bit: i32,

    // How far along the ray each axis's next cell face lies.
    next: vec3<f32>,

    // The axis of the face the ray last crossed, and how far along the ray
    // that face lies.
    across: vec3<bool>,
    along: f32,
}

// Whether the cell at `bit` is live in the tile whose words start at `tile`.
fn live(tile: u32, bit: i32) -> bool {
    return live_in(cells_at(tile, bit), bit);
}

// Moves `march` a cell along its ray. `rate` holds the ray's units per cell
// on each axis. `strides` holds what a step along each axis adds to a bit.
// Returns false once the ray has passed `end`.
//
// A march's loops count no steps. This test ends them, because each step
// moves a finite ray on by a cell. `sound` keeps any other ray from a
// fragment.
fn advance(march: ptr<function, March>, rate: vec3<f32>, strides: vec3<i32>, end: f32) -> bool {
    let next = (*march).next;
    let across = least(next);
    let along = min(next.x, min(next.y, next.z));

    (*march).across = across;
    (*march).along = along;
    (*march).next = next + select(vec3<f32>(0.0), rate, across);
    (*march).bit += select(select(strides.z, strides.y, across.y), strides.x, across.x);

    return along < end;
}

// The ray's direction through `box` at the rasterizer's point.
fn direction_of(box: Box) -> vec3<f32> {
    return box.point * RAY_W - box.ray;
}

// The first hit of the ray from `start` along `direction` in the box from
// `low` to `high` of the tile whose words start at `tile`. A hit lies
// `along` directions from `start`. A ray that `enters` starts on the face
// of the box across the one axis of `entry`. Any other ray starts inside
// the box, and live cells it starts in are no hit.
fn first_hit(
    tile: u32,
    low: vec3<f32>,
    high: vec3<f32>,
    start: vec3<f32>,
    direction: vec3<f32>,
    enters: bool,
    entry: vec3<bool>,
) -> Hit {
    let ahead = direction >= vec3<f32>(0.0);

    // The ray's units per cell on each axis.
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    let cell = clamp(floor(start), low, high - 1.0);

    // What a step along each axis adds to a cell's bit.
    let strides = select(vec3<i32>(-16, -256, -1), vec3<i32>(16, 256, 1), ahead);

    // How far along the ray each axis's next cell face lies.
    let within = start - cell;
    let next = abs(select(within, 1.0 - within, ahead)) * rate;

    // How far along the ray the box ends, short by more than the sums of
    // `next` can drift. A step that ends before it stays in the box.
    let ends = abs(select(low, high, ahead) - start) * rate;
    let end = min(ends.x, min(ends.y, ends.z)) * 0.99999;

    var march = March(bit_of(cell), next, entry, 0.0);

    // A ray that starts inside the box first leaves the live cells it
    // starts in.
    if !enters {
        loop {
            if !live(tile, march.bit) {
                break;
            }

            if !advance(&march, rate, strides, end) {
                return Hit(false, 0, entry, end, Cells());
            }
        }
    }

    loop {
        let cells = cells_at(tile, march.bit);

        if live_in(cells, march.bit) {
            return Hit(true, march.bit, march.across, march.along, cells);
        }

        if !advance(&march, rate, strides, end) {
            break;
        }
    }

    return Hit(false, 0, entry, end, Cells());
}

// The first hit of the ray that leaves the box at the rasterizer's point,
// which it lies `along` directions before. The ray starts where it enters
// the box, or at its origin when that lies inside the box.
fn hit_from_back(box: Box) -> Hit {
    let direction = direction_of(box);
    let ahead = direction >= vec3<f32>(0.0);
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    // How far back along the ray each axis's entry face lies, then the ray's
    // origin.
    let faces = abs(box.point - select(box.high, box.low, ahead)) * rate;
    let entry = min(faces.x, min(faces.y, faces.z));
    let origin = (1.0 - box.clip.z * (1.0 - RAY_W)) / (box.fall * (1.0 - RAY_W) + RAY_W);
    let before = min(entry, origin);

    var hit = first_hit(
        box.words.x,
        box.low,
        box.high,
        box.point - direction * before,
        direction,
        entry <= origin,
        least(faces),
    );

    hit.along -= before;

    return hit;
}

// The first hit of a perspective view's ray in `other`, from `base`
// directions past the eye and nearer than `reach` directions past that. A
// hit lies `along` directions past it. A miss holds `reach`.
fn other_hit(other: Other, solid: bool, base: f32, reach: f32) -> Hit {
    // Where the ray crosses each axis's two faces of the box, in directions
    // from the eye.
    let turned = select(other.direction, vec3<f32>(1.0e-12), abs(other.direction) < vec3<f32>(1.0e-12));
    let lows = (other.low - other.eye) / turned;
    let highs = (other.high - other.eye) / turned;
    let nears = min(lows, highs);
    let fars = max(lows, highs);
    let near = max(nears.x, max(nears.y, nears.z));
    let far = min(fars.x, min(fars.y, fars.z));

    // The ray's stretch in both boxes starts `base` past the eye or where
    // the ray enters the other box, whichever comes later.
    let first = max(near, base);

    // A ray that starts the stretch inside a solid box met its face before
    // the stretch, where another box finds it, or starts at an eye inside
    // the box, which sees none of it.
    if first >= min(far, base + reach) || (solid && near < base) {
        return Hit(false, 0, vec3<bool>(false), reach, Cells());
    }

    let start = other.eye + other.direction * first;
    let entry = least(-nears);
    let bit = bit_of(clamp(floor(start), other.low, other.high - 1.0));
    var hit = Hit(true, bit, entry, 0.0, Cells());

    if solid {
        hit.cells = cells_at(other.tile, bit);
    } else {
        // A ray from an eye inside the box takes no hit in the live cells
        // it starts in.
        let enters = base > 0.0 || near >= 0.0;

        hit = first_hit(other.tile, other.low, other.high, start, other.direction, enters, entry);
    }

    hit.along += first - base;

    if !hit.found || hit.along >= reach {
        return Hit(false, 0, vec3<bool>(false), reach, Cells());
    }

    return hit;
}

// What a fragment writes for `hit` in `source`. The hit lies `hit.along`
// directions past `start` in the source's tile.
fn shade(source: Source, hit: Hit, start: vec3<f32>, direction: vec3<f32>) -> vec4<f32> {
    let cell = vec3<i32>((hit.bit >> 4u) & 15, hit.bit >> 8u, hit.bit & 15);

    return surface(
        source,
        hit.cells,
        cell,
        hit.across,
        direction >= vec3<f32>(0.0),
        start + direction * hit.along,
        direction,
    );
}

// The one axis of the fragment's face.
fn face_of(box: Box) -> vec3<bool> {
    return vec3<u32>(box.face & 3u) == vec3<u32>(0u, 1u, 2u);
}

// A front face's fragment. The rasterizer's point is where the ray enters
// the box.
//
// The march runs in the entry's own loop and shades where it hits. Fewer
// values then stay live through the loop than `first_hit` hands back, and
// the fragment ran 8% faster on Quest.
@fragment
fn front(box: Box) -> @location(0) vec4<f32> {
    let direction = direction_of(box);
    let ahead = direction >= vec3<f32>(0.0);
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));
    let cell = clamp(floor(box.point), box.low, box.high - 1.0);
    let strides = select(vec3<i32>(-16, -256, -1), vec3<i32>(16, 256, 1), ahead);
    let within = box.point - cell;
    let next = abs(select(within, 1.0 - within, ahead)) * rate;
    let ends = abs(select(box.low, box.high, ahead) - box.point) * rate;
    let end = min(ends.x, min(ends.y, ends.z)) * 0.99999;

    var march = March(bit_of(cell), next, face_of(box), 0.0);

    loop {
        let cells = cells_at(box.words.x, march.bit);

        if live_in(cells, march.bit) {
            let hit = Hit(true, march.bit, march.across, march.along, cells);

            return shade(source_of(box), hit, box.point, direction);
        }

        if !advance(&march, rate, strides, end) {
            break;
        }
    }

    discard;
}

// A front face's fragment of a box that shares space with other placements'
// boxes, under a perspective view.
@fragment
fn crossed(box: Box) -> @location(0) vec4<f32> {
    let direction = direction_of(box);
    let turned = select(direction, vec3<f32>(1.0e-12), abs(direction) < vec3<f32>(1.0e-12));

    var hit = first_hit(box.words.x, box.low, box.high, box.point, direction, true, face_of(box));
    var source = source_of(box);

    // The fragment's point and the ray's direction in the hit's tile.
    var start = box.point;
    var along = direction;

    for (var n = 0u; n < box.crossers.y; n++) {
        let low = crossers[(box.crossers.x + n) * 2u];
        let high = crossers[(box.crossers.x + n) * 2u + 1u];

        // Where the ray crosses the space the two boxes share, in
        // directions past the fragment's point.
        let lows = (low.xyz - box.point) / turned;
        let highs = (high.xyz - box.point) / turned;
        let nears = min(lows, highs);
        let fars = max(lows, highs);
        let near = max(nears.x, max(nears.y, nears.z));
        let far = min(fars.x, min(fars.y, fars.z));

        // The other box can hold a nearer hit only where the ray crosses
        // that space before its hit so far.
        if max(near, 0.0) >= min(far, hit.along) {
            continue;
        }

        let crosser = bitcast<u32>(low.w);
        let other = other_of(box, direction, crosser & ~SOLID);
        let nearer = other_hit(other, (crosser & SOLID) != 0u, 1.0, hit.along);

        if nearer.found {
            hit = nearer;
            source = other.source;
            start = other.eye + other.direction;
            along = other.direction;
        }
    }

    if !hit.found {
        discard;
    }

    return shade(source, hit, start, along);
}

// A back face's fragment of a box that holds a perspective view's eye and
// shares space with other placements' boxes.
@fragment
fn held(box: Box) -> @location(0) vec4<f32> {
    let direction = direction_of(box);
    let turned = select(direction, vec3<f32>(1.0e-12), abs(direction) < vec3<f32>(1.0e-12));
    let ahead = direction >= vec3<f32>(0.0);
    let rate = 1.0 / max(abs(direction), vec3<f32>(1.0e-12));

    // How far back along the ray each axis's entry face lies. The ray
    // starts where it enters the box, or at the eye inside the box, one
    // direction back.
    let faces = abs(box.point - select(box.high, box.low, ahead)) * rate;
    let entry = min(faces.x, min(faces.y, faces.z));
    let before = min(entry, 1.0);
    let begin = box.point - direction * before;

    var hit = first_hit(box.words.x, box.low, box.high, begin, direction, entry <= 1.0, least(faces));
    var source = source_of(box);

    // The ray's start and its direction in the hit's tile.
    var start = begin;
    var along = direction;

    for (var n = 0u; n < box.crossers.y; n++) {
        let low = crossers[(box.crossers.x + n) * 2u];
        let high = crossers[(box.crossers.x + n) * 2u + 1u];

        // Where the ray crosses the space the two boxes share, in
        // directions past its start.
        let lows = (low.xyz - begin) / turned;
        let highs = (high.xyz - begin) / turned;
        let nears = min(lows, highs);
        let fars = max(lows, highs);
        let near = max(nears.x, max(nears.y, nears.z));
        let far = min(fars.x, min(fars.y, fars.z));

        if max(near, 0.0) >= min(far, hit.along) {
            continue;
        }

        let crosser = bitcast<u32>(low.w);
        let other = other_of(box, direction, crosser & ~SOLID);
        let nearer = other_hit(other, (crosser & SOLID) != 0u, 1.0 - before, hit.along);

        if nearer.found {
            hit = nearer;
            source = other.source;
            start = other.eye + other.direction * (1.0 - before);
            along = other.direction;
        }
    }

    if !hit.found {
        discard;
    }

    return shade(source, hit, start, along);
}

// A back face's fragment of a box a ray can start inside.
@fragment
fn inside(box: Box) -> @location(0) vec4<f32> {
    let hit = hit_from_back(box);

    if !hit.found {
        discard;
    }

    return shade(source_of(box), hit, box.point, direction_of(box));
}

// A back face's fragment that writes its hit's depth. A depth of zero fails
// the depth test, so a miss writes nothing.
@fragment
fn exact(box: Box) -> Fragment {
    let hit = hit_from_back(box);

    if !hit.found {
        return Fragment(vec4<f32>(0.0), 0.0);
    }

    return Fragment(
        shade(source_of(box), hit, box.point, direction_of(box)),
        (box.clip.z - hit.along * box.fall * (1.0 - RAY_W)) / (1.0 + hit.along * RAY_W),
    );
}

// A solid box's fragment. The rasterizer's point lies on a live cell's face.
@fragment
fn solid(box: Box) -> @location(0) vec4<f32> {
    let cell = clamp(floor(box.point), box.low, box.high - 1.0);

    // A ray meets a box's low face moving up the face's axis.
    return surface(
        source_of(box),
        cells_at(box.words.x, bit_of(cell)),
        vec3<i32>(cell),
        face_of(box),
        vec3<bool>(box.face < 4u),
        box.point,
        direction_of(box),
    );
}
