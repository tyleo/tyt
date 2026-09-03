use vmax::VMaxSceneCamera;

/// Overrides the scene camera a written Voxel Max document opens with. Voxel
/// Max presents a document with no selected object through its scene camera.
/// Every document this crate synthesizes has no selection, so the override
/// picks the view such a document opens to. Without one the document keeps
/// the camera its path produces: the ext's in the lossless path, the empty
/// default in synthesis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SceneCameraSource {
    /// The ext's scene camera; errors when the state carries no `voxel-max`
    /// ext.
    Ext,
    /// The empty default scene camera, replacing any the ext carries.
    Empty,
    /// The given scene camera, replacing any the ext carries.
    Camera(VMaxSceneCamera),
}
