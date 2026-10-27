use ty_math::TyVector3I32;

/// The offsets from a cell to its six face neighbors.
pub const FACE_OFFSETS: [TyVector3I32; 6] = [
    TyVector3I32::new(-1, 0, 0),
    TyVector3I32::new(1, 0, 0),
    TyVector3I32::new(0, -1, 0),
    TyVector3I32::new(0, 1, 0),
    TyVector3I32::new(0, 0, -1),
    TyVector3I32::new(0, 0, 1),
];
