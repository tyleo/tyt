use crate::{Error, Result, TileScene, model_rig};
use std::{fs, mem, path::Path};
use ty_math::TyBoundsF64;
use voxj::{VoxjPositionBlock, VoxjSampleBlock};
use voxj_codec::{DependenciesImpl, from_voxj_or_voxjz_file_bytes};
use voxj_voxcore::from_voxj_file;
use voxrender::{RenderBloom, RenderScene, RenderView};

/// A Voxel Json model as a [`BoxView`](crate::BoxView) draws it, with what
/// frames it.
pub struct ModelTiles {
    /// Every object at its nodes' placements, in meters, under vxl's
    /// studio rig as [`view`](Self::view) places the rig.
    pub tiles: TileScene,

    /// The world bounds of the model's live voxels.
    pub bounds: TyBoundsF64,

    /// vxl's hero view of the model for a square image.
    pub view: RenderView,

    /// The bloom vxl's review renders take.
    pub bloom: RenderBloom,
}

impl ModelTiles {
    /// Loads the document at `path` an object at a time. The loader holds
    /// one object's grid while the scene keeps the tiles of the rest, so a
    /// document can hold more cells than the device has room for. With
    /// `glass`, the scene keeps the surfaces between materials that pass
    /// light. Errors if the document does not load, holds no live voxel,
    /// or holds a material the contract cannot shade, and as
    /// [`TileScene::push_scene`] does.
    pub fn load(path: &Path, glass: bool) -> Result<Self> {
        let model = |error: String| Error::Model {
            path: path.to_owned(),
            error,
        };

        let bytes = fs::read(path).map_err(|error| model(error.to_string()))?;

        let mut file = from_voxj_or_voxjz_file_bytes(&DependenciesImpl, &bytes)
            .map_err(|error| model(error.to_string()))?;

        drop(bytes);

        // Each object's cells leave the document, and one object's at a
        // time come back for the document's conversion. An object without
        // them converts to an empty grid.
        let mut held: Vec<_> = file
            .main
            .runtime_state
            .objects
            .iter_mut()
            .map(|object| {
                let layers = object.layers.len();

                (
                    mem::take(&mut object.bounds),
                    mem::replace(
                        &mut object.voxel_positions,
                        VoxjPositionBlock::RawJson(Vec::new()),
                    ),
                    mem::replace(
                        &mut object.voxel_samples,
                        VoxjSampleBlock::RawJson(vec![Vec::new(); layers]),
                    ),
                )
            })
            .collect();

        // An object's build volume sizes its grid where the document
        // holds one.
        let mut volumes: Vec<_> = file
            .main
            .edit_state
            .iter_mut()
            .flat_map(|edit| &mut edit.objects)
            .map(|object| mem::take(&mut object.bounds))
            .collect();

        let mut tiles = TileScene::new(glass);
        let mut bounds: Option<TyBoundsF64> = None;

        for index in 0..held.len() {
            let swap = |file: &mut voxj::VoxjFile,
                        held: &mut Vec<([u32; 3], VoxjPositionBlock, VoxjSampleBlock)>,
                        volumes: &mut Vec<[u32; 3]>| {
                let object = &mut file.main.runtime_state.objects[index];
                let (bounds, positions, samples) = &mut held[index];

                mem::swap(&mut object.bounds, bounds);
                mem::swap(&mut object.voxel_positions, positions);
                mem::swap(&mut object.voxel_samples, samples);

                if let Some(edit) = &mut file.main.edit_state
                    && let Some(object) = edit.objects.get_mut(index)
                {
                    mem::swap(&mut object.bounds, &mut volumes[index]);
                }
            };

            swap(&mut file, &mut held, &mut volumes);

            let main =
                from_voxj_file(&DependenciesImpl, &file).map_err(|error| model(error.to_string()));

            swap(&mut file, &mut held, &mut volumes);

            let main = main?;

            let (object_id, _) = main
                .iter_objects()
                .nth(index)
                .expect("the conversion keeps the document's objects in order");

            let scene =
                RenderScene::from_vox_main(&main, &[object_id], 1.0).map_err(Error::Render)?;

            drop(main);

            let placement_ids: Vec<_> = scene.iter_placements().map(|(id, _)| id).collect();

            let placed = scene
                .subject_bounds(&placement_ids)
                .expect("the placements are the scene's");

            if let Some(placed) = placed {
                bounds = Some(bounds.map_or(placed, |bounds| bounds.encapsulate(&placed)));
            }

            tiles.push_scene(&scene)?;
        }

        let bounds = bounds.ok_or_else(|| model("the document holds no live voxel".to_owned()))?;
        let (view, lights, bloom) = model_rig(&bounds)?;

        tiles.set_lights(lights.to_vec());

        Ok(ModelTiles {
            tiles,
            bounds,
            view,
            bloom,
        })
    }
}
