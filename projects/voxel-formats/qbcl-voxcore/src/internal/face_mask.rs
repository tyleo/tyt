use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::{BVoxVoxel, VoxObject};

/// The left face, toward `-x`, shows.
const LEFT: u8 = 2;

/// The right face, toward `+x`, shows.
const RIGHT: u8 = 4;

/// The top face, toward `+y`, shows.
const TOP: u8 = 8;

/// The bottom face, toward `-y`, shows.
const BOTTOM: u8 = 16;

/// The front face, toward `-z`, shows. Qubicle's frame is left-handed with
/// `z` growing away from the viewer, so the front of a voxel is its low `z`
/// side.
const FRONT: u8 = 32;

/// The back face, toward `+z`, shows.
const BACK: u8 = 64;

/// The Qubicle visibility mask of the live voxel `voxel_id` in `object`: a
/// face's bit is set when the neighbor cell in that direction is outside the
/// grid or empty. The bits follow the `.qb` spec.
pub(crate) fn face_mask(object: &VoxObject, voxel_id: U32Id<BVoxVoxel>) -> u8 {
    let position = object
        .voxel_position(voxel_id)
        .expect("a live voxel is within the grid");

    let covered = |dx: i64, dy: i64, dz: i64| {
        let x = i64::from(position.x) + dx;
        let y = i64::from(position.y) + dy;
        let z = i64::from(position.z) + dz;
        let (Ok(x), Ok(y), Ok(z)) = (u32::try_from(x), u32::try_from(y), u32::try_from(z)) else {
            return false;
        };
        let Some(neighbor_id) = object.voxel_id(TyVector3U32::new(x, y, z)) else {
            return false;
        };
        object.is_live(neighbor_id)
    };

    let faces = [
        (LEFT, (-1, 0, 0)),
        (RIGHT, (1, 0, 0)),
        (TOP, (0, 1, 0)),
        (BOTTOM, (0, -1, 0)),
        (FRONT, (0, 0, -1)),
        (BACK, (0, 0, 1)),
    ];

    let mut mask = 0;
    for (bit, (dx, dy, dz)) in faces {
        if !covered(dx, dy, dz) {
            mask |= bit;
        }
    }

    // Qubicle reads a zero mask as an empty cell, so a voxel every neighbor
    // covers keeps every face bit and the renderer culls.
    if mask == 0 {
        mask = LEFT | RIGHT | TOP | BOTTOM | FRONT | BACK;
    }

    mask
}

#[cfg(test)]
mod tests {
    use crate::face_mask;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxMaterial, BVoxPalette, VoxObject};

    /// A `3x3x3` object with the listed cells live.
    fn object(live: &[[u32; 3]]) -> VoxObject {
        let mut object = VoxObject::new(String::new(), TyVector3U32::new(3, 3, 3)).unwrap();

        object.retain_layer(
            U32Id::<BVoxPalette>::from_u32(0),
            U32Id::<BVoxMaterial>::from_u32(0),
        );

        for &[x, y, z] in live {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, y, z)).unwrap();

            object
                .retain_voxel(voxel_id, &[U32Id::from_u32(0)])
                .unwrap();
        }

        object
    }

    fn mask_at(object: &VoxObject, x: u32, y: u32, z: u32) -> u8 {
        face_mask(object, object.voxel_id(TyVector3U32::new(x, y, z)).unwrap())
    }

    #[test]
    fn a_lone_voxel_shows_every_face() {
        let object = object(&[[1, 1, 1]]);

        assert_eq!(mask_at(&object, 1, 1, 1), 0x7e);
    }

    #[test]
    fn a_neighbor_covers_the_face_toward_it() {
        let object = object(&[[0, 1, 1], [1, 1, 1]]);

        // The left voxel's right face is covered, the right voxel's left face.
        assert_eq!(mask_at(&object, 0, 1, 1), 0x7e & !4);

        assert_eq!(mask_at(&object, 1, 1, 1), 0x7e & !2);
    }

    #[test]
    fn each_axis_maps_to_its_pair_of_bits() {
        let object = object(&[[1, 1, 1], [1, 2, 1], [1, 1, 2]]);

        // Covered above and behind.
        assert_eq!(mask_at(&object, 1, 1, 1), 0x7e & !8 & !64);

        assert_eq!(mask_at(&object, 1, 2, 1), 0x7e & !16);

        assert_eq!(mask_at(&object, 1, 1, 2), 0x7e & !32);
    }

    #[test]
    fn a_fully_covered_voxel_keeps_a_solid_mask() {
        let mut live = Vec::new();

        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    live.push([x, y, z]);
                }
            }
        }

        let object = object(&live);

        assert_eq!(mask_at(&object, 1, 1, 1), 0x7e);

        assert_eq!(mask_at(&object, 0, 0, 0), 2 | 16 | 32);
    }
}
