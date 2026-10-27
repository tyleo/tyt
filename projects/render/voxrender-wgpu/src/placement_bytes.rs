use crate::push_i32s;
use voxrender::{GRID_GUARD_BITS, RenderGridViewRays, RenderScene};

/// The bytes of `grid_walk.wgsl`'s `Placement` for each of `scene`'s
/// placements, in placement order.
///
/// # Arguments
///
/// 1. `grid_rays`: each placement's rays of one view
/// 2. `cell_offsets`: the offset of each placement's object in the words the
///    walk reads
pub fn placement_bytes(
    scene: &RenderScene,
    grid_rays: &[RenderGridViewRays],
    cell_offsets: &[u32],
) -> Vec<u8> {
    assert_eq!(
        grid_rays.len(),
        cell_offsets.len(),
        "each placement has its rays and its cell offset"
    );

    let mut bytes = Vec::new();

    for (((_, placement), rays), &offset) in
        scene.iter_placements().zip(grid_rays).zip(cell_offsets)
    {
        let object = scene
            .object(placement.object_id)
            .expect("a placement's object is one of the scene's");
        let bounds = object
            .bounds()
            .to_array()
            .map(|side| i32::try_from(side).expect("a grid's sides are within range"));
        let scale = placement
            .transform
            .scale
            .to_array()
            .map(|scale| scale as f32);

        let rotation = placement.transform.rotation;

        push_i32s(&mut bytes, bounds);
        push_u32s(&mut bytes, [offset, 0, 0]);
        push_f32s(&mut bytes, scale);
        push_f32s(
            &mut bytes,
            placement
                .transform
                .position
                .to_array()
                .map(|coordinate| coordinate as f32),
        );

        for value in [rotation.x, rotation.y, rotation.z, rotation.w] {
            bytes.extend((value as f32).to_le_bytes());
        }

        for steps in [
            [rays.origin, rays.origin_x, rays.origin_y],
            [rays.direction, rays.direction_x, rays.direction_y],
        ] {
            for vector in steps {
                let (whole, fraction) = split(vector);

                push_i32s(&mut bytes, whole);
                push_i32s(&mut bytes, fraction);
            }
        }
    }

    bytes
}

/// `vector`, which carries guard bits, as whole units and a fraction in
/// `[0, 2^GRID_GUARD_BITS)`, so the shader can sum steps in i32.
fn split(vector: [i64; 3]) -> ([i32; 3], [i32; 3]) {
    let mask = (1 << GRID_GUARD_BITS) - 1;

    (
        vector.map(|component| {
            i32::try_from(component >> GRID_GUARD_BITS).expect("an in-range ray's steps fit i32")
        }),
        vector.map(|component| (component & mask) as i32),
    )
}

/// Appends `values` and a zero as one `vec4<u32>`.
fn push_u32s(bytes: &mut Vec<u8>, values: [u32; 3]) {
    for value in values.into_iter().chain([0]) {
        bytes.extend(value.to_le_bytes());
    }
}

/// Appends `values` and a zero as one `vec4<f32>`.
fn push_f32s(bytes: &mut Vec<u8>, values: [f32; 3]) {
    for value in values.into_iter().chain([0.0]) {
        bytes.extend(value.to_le_bytes());
    }
}

#[cfg(test)]
mod tests {
    use crate::placement_bytes::split;

    #[test]
    fn a_split_keeps_its_value_with_a_fraction_under_the_guard() {
        let vector = [(1 << 40) + 4095, -1, -(1 << 40) - 4096];
        let (whole, fraction) = split(vector);

        assert_eq!(whole, [1 << 28, -1, -(1 << 28) - 1]);
        assert_eq!(fraction, [4095, 4095, 0]);

        for a in 0..3 {
            assert_eq!(
                (i64::from(whole[a]) << 12) + i64::from(fraction[a]),
                vector[a]
            );
        }
    }
}
