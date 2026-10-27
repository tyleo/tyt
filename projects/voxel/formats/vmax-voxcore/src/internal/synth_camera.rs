use vmax::VMaxSceneCamera;

/// The scene camera a synthesized document opens with, mirroring a fresh Voxel
/// Max document's neutral rig. Voxel Max needs a valid rig to present an
/// imported document; the framing is cosmetic.
pub const SYNTH_CAMERA: VMaxSceneCamera = VMaxSceneCamera {
    da: 0.0,
    ha: 0.25,
    lda: 0.0,
    lha: 1.875,
    lwa: 0.25,
    o: [0.0, 0.0, 0.0],
    px: 0.0,
    py: 0.0,
    wa: 0.0,
    z: 512.0,
    aq: None,
    op: None,
    zf: None,
};
