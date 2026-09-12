use vmax::VMaxViewBox;

/// An object's internal-grid placement derived for the Voxel Max write: where
/// its voxels sit in the private workspace, and the content box and build
/// volume that follow. The scene placement is recovered separately from the
/// node transform.
pub struct ObjectPlacement {
    /// The runtime grid's min corner in the workspace.
    pub box_min: [i32; 3],

    /// The grid `origin` carried over from the object, for the pivot math.
    pub origin: [i32; 3],

    /// The content center (`e_c`): `box_min + bounds / 2`.
    pub center: [f64; 3],

    /// The content box min relative to the center (`e_mi`): `-bounds / 2`.
    pub bounds_min: [f64; 3],

    /// The content box max relative to the center (`e_ma`): `bounds / 2`.
    pub bounds_max: [f64; 3],

    /// The build volume (`tools.vp`): the edit grid placed in the workspace.
    pub view_box: VMaxViewBox,
}
