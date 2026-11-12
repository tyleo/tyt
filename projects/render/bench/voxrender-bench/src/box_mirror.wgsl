// What a face's mirror ray meets, through a hardware ray query over the
// scene's opaque quads. `box_view.wgsl` binds what it reads.

// Per quad of each object: its face, the axis doubled plus one where the
// face looks up the axis, under its material's place in `materials` from
// bit 3 up. An instance's custom data holds its object's first quad.
@group(0) @binding(7) var<storage, read> quads: array<u32>;
@group(0) @binding(10) var tlas: acceleration_structure;

// The light that reaches `origin` from the quad the ray along the unit
// `direction` meets within `reach` meters, then the meters to the quad.
// The meters are negative where the ray meets none. The quad shades flat:
// its material's emission, and its color under the sun and the hemispheres
// with no shadow and no occlusion.
fn mirrored(origin: vec3<f32>, direction: vec3<f32>, reach: f32) -> vec4<f32> {
    var query: ray_query;

    rayQueryInitialize(&query, tlas, RayDesc(RAY_FLAG_NONE, 0xffu, 0.0, reach, origin, direction));

    // An opaque triangle commits its hit as the query proceeds.
    while rayQueryProceed(&query) {}

    let hit = rayQueryGetCommittedIntersection(&query);

    if hit.kind == RAY_QUERY_INTERSECTION_NONE {
        return vec4<f32>(-1.0);
    }

    let quad = quads[hit.instance_custom_data + hit.primitive_index / 2u];
    let face = quad & 7u;
    let material = quad >> 3u;
    let base = materials[material * 3u];
    let glow = materials[material * 3u + 1u];
    let gloss = materials[material * 3u + 2u];

    // The face's normal in the world.
    let turn = hit.object_to_world;
    let axis = normalize(select(select(turn[2], turn[1], face >> 1u == 1u), turn[0], face >> 1u == 0u));
    let normal = select(-axis, axis, (face & 1u) == 1u);

    let ambient = sky_light(normal, 1.0);
    let sun = frame.sunlight.rgb * (max(dot(normal, frame.sun.xyz), 0.0) / 3.14159265);

    return vec4<f32>(
        glow.rgb + base.rgb * ((1.0 - base.a) * (ambient + sun)) + gloss.rgb * ambient,
        hit.t,
    );
}
