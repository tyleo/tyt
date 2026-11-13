use crate::{Error, Result, model_rig};
use std::path::Path;
use voxconv::{DependenciesImpl, ReadFormat, load};
use voxrender::{RenderBloom, RenderScene};

/// The scene of the Voxel Json document at `path`: every object at its
/// nodes' placements, in meters. The scene's first view is vxl's hero view
/// of the model for a square image. Its lights are vxl's studio rig as that
/// view places the rig. The bloom is the one vxl's review renders take.
/// Errors if the document does not load, holds no live voxel, or holds a
/// material the contract cannot shade.
pub fn model_scene(path: &Path) -> Result<(RenderScene, RenderBloom)> {
    let model = |error: String| Error::Model {
        path: path.to_owned(),
        error,
    };

    let main = load(&DependenciesImpl, ReadFormat::Voxj, path)
        .map_err(|error| model(error.to_string()))?;

    let object_ids: Vec<_> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .collect();

    let mut scene = RenderScene::from_vox_main(&main, &object_ids, 1.0).map_err(Error::Render)?;

    let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();

    let bounds = scene
        .subject_bounds(&placement_ids)
        .expect("the placements are the scene's")
        .ok_or_else(|| model("the document holds no live voxel".to_owned()))?;

    let (view, lights, bloom) = model_rig(&bounds)?;

    scene.retain_view(view).map_err(Error::Render)?;

    for light in lights {
        scene.retain_light(light).map_err(Error::Render)?;
    }

    Ok((scene, bloom))
}
