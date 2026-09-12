use vmax::VMaxSceneCamera;

/// The scene camera a written Voxel Max document opens with. Voxel Max
/// presents a document with no selected object through its scene camera.
/// Every document this crate synthesizes has no selection, so the choice
/// picks the view such a document opens to.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SceneCameraSource {
    /// The ext's scene camera, the neutral default for a synthesized ext.
    #[default]
    Ext,
    /// The neutral default scene camera, replacing any the ext carries.
    Empty,
    /// The given scene camera, replacing any the ext carries.
    Camera(VMaxSceneCamera),
}
