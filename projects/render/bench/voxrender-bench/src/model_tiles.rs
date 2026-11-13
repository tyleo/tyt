use crate::{
    CellGrid, Error, ObjectTiles, Result, TileScene, load_threads, model_rig, object_copies,
};
use std::{
    collections::HashMap,
    fs, mem,
    path::Path,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
};
use ty_math::{TyBoundsF64, TyTransformF64, TyVector3U32};
use voxj::{VoxjFile, VoxjPositionBlock, VoxjSampleBlock};
use voxj_codec::{DependenciesImpl, from_voxj_or_voxjz_file_bytes};
use voxj_voxcore::from_voxj_file;
use voxrender::{RenderBloom, RenderScene, RenderView};

/// One object's cells as a document holds them: its bounds, its positions,
/// its samples, and the bounds of its build volume where the document has
/// one.
type Held = (
    [u32; 3],
    VoxjPositionBlock,
    VoxjSampleBlock,
    Option<[u32; 3]>,
);

/// One object as a thread builds it: its tiles and those of its coarser
/// copies, the empty cells ahead of its first, its placements, and the
/// world bounds of its live voxels.
type Built = (
    Vec<ObjectTiles>,
    TyVector3U32,
    Vec<TyTransformF64>,
    Option<TyBoundsF64>,
);

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
    /// Loads the document at `path` an object at a time on several
    /// threads. A thread holds one object's grid, and the scene keeps the
    /// tiles of the rest, so a document can hold more cells than the device
    /// has room for. The scene takes `glass` and `coarser` as
    /// [`TileScene::new`] does. Errors if the document does not load, holds
    /// no live voxel, or holds a material the contract cannot shade, and as
    /// [`TileScene::push_object`] does.
    pub fn load(path: &Path, glass: bool, coarser: bool) -> Result<Self> {
        let model = |error: String| Error::Model {
            path: path.to_owned(),
            error,
        };

        let bytes = fs::read(path).map_err(|error| model(error.to_string()))?;

        let mut file = from_voxj_or_voxjz_file_bytes(&DependenciesImpl, &bytes)
            .map_err(|error| model(error.to_string()))?;

        drop(bytes);

        // Each object's cells leave the document. One object's at a time
        // come back to a thread's copy of it for the document's
        // conversion, where an object without them converts to an empty
        // grid. An object's build volume sizes its grid where the document
        // holds one.
        let mut volumes: Vec<_> = (file.main.edit_state.iter_mut())
            .flat_map(|edit| &mut edit.objects)
            .map(|object| mem::take(&mut object.bounds))
            .collect();

        volumes.reverse();

        let held: Vec<Option<Held>> = (file.main.runtime_state.objects.iter_mut())
            .map(|object| {
                let layers = object.layers.len();

                Some((
                    mem::take(&mut object.bounds),
                    mem::replace(
                        &mut object.voxel_positions,
                        VoxjPositionBlock::RawJson(Vec::new()),
                    ),
                    mem::replace(
                        &mut object.voxel_samples,
                        VoxjSampleBlock::RawJson(vec![Vec::new(); layers]),
                    ),
                    volumes.pop(),
                ))
            })
            .collect();

        let count = held.len();
        let held = Mutex::new(held);

        // The scene with the object whose materials it takes next. The
        // objects place their materials in the document's order, so a
        // material keeps its place however the threads run.
        let scene = Mutex::new((TileScene::new(glass, coarser), 0));
        let turn = Condvar::new();

        let built: Mutex<Vec<Option<Built>>> = Mutex::new((0..count).map(|_| None).collect());
        let failed: Mutex<Option<Error>> = Mutex::new(None);
        let stopped = AtomicBool::new(false);
        let next = AtomicUsize::new(0);

        thread::scope(|scope| {
            for _ in 0..load_threads() {
                scope.spawn(|| {
                    let mut file = file.clone();

                    while !stopped.load(Ordering::Relaxed) {
                        let index = next.fetch_add(1, Ordering::Relaxed);

                        if index >= count {
                            break;
                        }

                        let cells = held.lock().expect("no thread panics with the lock")[index]
                            .take()
                            .expect("each object's cells leave once");

                        let converted = object_scene(path, &mut file, index, cells);

                        // Each object takes its turn, so the one after it
                        // never waits on an object that failed.
                        let placed = {
                            let mut scene = scene.lock().expect("no thread panics with the lock");

                            while scene.1 != index {
                                scene = turn.wait(scene).expect("no thread panics with the lock");
                            }

                            let placed = converted.map(|object_scene| {
                                let places: HashMap<u32, u32> = (object_scene.iter_materials())
                                    .map(|(id, material)| {
                                        (id.to_u32(), scene.0.place_material(material))
                                    })
                                    .collect();

                                let (opaque, glass) =
                                    (scene.0.opaque.clone(), scene.0.glass_materials.clone());

                                (object_scene, places, opaque, glass)
                            });

                            scene.1 += 1;
                            turn.notify_all();

                            placed
                        };

                        let object =
                            placed.and_then(|(object_scene, places, opaque, glass_materials)| {
                                let (object_id, object) = (object_scene.iter_objects().next())
                                    .expect("the scene holds the document's object");

                                let grid = CellGrid::new(object, &|id| places[&id]);
                                let first = TileScene::first_cell(&object_scene, object_id);

                                let placement_ids: Vec<_> =
                                    object_scene.iter_placements().map(|(id, _)| id).collect();

                                let bounds = object_scene
                                    .subject_bounds(&placement_ids)
                                    .expect("the placements are the scene's");

                                let transforms = (object_scene.iter_placements())
                                    .map(|(_, placement)| placement.transform)
                                    .collect();

                                drop(object_scene);

                                let glass = glass.then_some(glass_materials.as_slice());
                                let (copies, shift) =
                                    object_copies(&grid, first, coarser, &opaque, glass)?;

                                Ok((copies, shift, transforms, bounds))
                            });

                        match object {
                            Ok(object) => {
                                built.lock().expect("no thread panics with the lock")[index] =
                                    Some(object);
                            }

                            Err(error) => {
                                stopped.store(true, Ordering::Relaxed);
                                failed
                                    .lock()
                                    .expect("no thread panics with the lock")
                                    .get_or_insert(error);
                            }
                        }
                    }
                });
            }
        });

        if let Some(error) = failed.into_inner().expect("the threads ended") {
            return Err(error);
        }

        let (mut tiles, _) = scene.into_inner().expect("the threads ended");
        let mut bounds: Option<TyBoundsF64> = None;

        for object in built.into_inner().expect("the threads ended") {
            let (copies, shift, transforms, placed) = object.expect("each object built");
            let at = tiles.push_copies(copies, shift);

            for transform in transforms {
                tiles.place_object(at, transform);
            }

            if let Some(placed) = placed {
                bounds = Some(bounds.map_or(placed, |bounds| bounds.encapsulate(&placed)));
            }
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

/// The scene of the object at `index` of `file`'s objects alone, once the
/// object takes `cells`. `file` holds the document at `path` with every
/// object lacking its cells, and holds it so again on return. Errors if the
/// document does not convert or holds a material the contract cannot shade.
fn object_scene(
    path: &Path,
    file: &mut VoxjFile,
    index: usize,
    cells: Held,
) -> Result<RenderScene> {
    let (bounds, positions, samples, volume) = cells;
    let layers = file.main.runtime_state.objects[index].layers.len();

    let edit = |file: &mut VoxjFile, bounds: [u32; 3]| {
        if let Some(edit) = &mut file.main.edit_state
            && let Some(object) = edit.objects.get_mut(index)
        {
            object.bounds = bounds;
        }
    };

    let object = &mut file.main.runtime_state.objects[index];

    object.bounds = bounds;
    object.voxel_positions = positions;
    object.voxel_samples = samples;
    edit(file, volume.unwrap_or_default());

    let main = from_voxj_file(&DependenciesImpl, file);
    let object = &mut file.main.runtime_state.objects[index];

    object.bounds = [0; 3];
    object.voxel_positions = VoxjPositionBlock::RawJson(Vec::new());
    object.voxel_samples = VoxjSampleBlock::RawJson(vec![Vec::new(); layers]);
    edit(file, [0; 3]);

    let main = main.map_err(|error| Error::Model {
        path: path.to_owned(),
        error: error.to_string(),
    })?;

    let (object_id, _) = main
        .iter_objects()
        .nth(index)
        .expect("the conversion keeps the document's objects in order");

    RenderScene::from_vox_main(&main, &[object_id], 1.0).map_err(Error::Render)
}
