use ty_math::TyVector3F64;

/// The corners of the box from `low` to `high`. Bit `a` of a corner's index
/// picks the box's high side on axis `a`.
pub fn box_corners(low: TyVector3F64, high: TyVector3F64) -> [TyVector3F64; 8] {
    [0, 1, 2, 3, 4, 5, 6, 7].map(|corner: usize| {
        let pick = |axis: usize| [low, high][corner >> axis & 1][axis];

        TyVector3F64::new(pick(0), pick(1), pick(2))
    })
}
