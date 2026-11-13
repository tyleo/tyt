use crate::{
    BoxView, Error, GalleryControls, HeadsetEye, HeadsetStand, HeadsetView, ModelTiles, Result,
};
use std::{
    f64::consts::FRAC_PI_6,
    fs,
    io::Result as IOResult,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Instant,
};
use ty_math::{TyBoundsF64, TyQuaternionF64, TyVector3F64};
use wgpu::{Color, CommandEncoder, Device, Queue, RenderPass, TextureFormat};

/// The meters a second a full stick moves the wearer, in the headset's
/// meters.
const SPEED: f64 = 1.5;

/// The radians one lean of the turn stick turns the wearer.
const SNAP: f64 = FRAC_PI_6;

/// How far the turn stick leans before it turns the wearer, and how far
/// back it comes before it can turn them again.
const LEAN: [f32; 2] = [0.7, 0.3];

/// A stick's travel from its center that reads as none.
const DEAD: f32 = 0.15;

/// The times a model grows in a second under a full trigger.
const GROWTH: f64 = 4.0;

/// The meters ahead of the head to the point a growing model holds still
/// around.
const HELD: f64 = 1.0;

/// The color the eyes clear to before the first model lands.
const EMPTY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

/// The share of its light the sky keeps while a model loads.
const DIM: f64 = 0.7;

/// The Voxel Json models under a directory, shown one at a time around a
/// headset's wearer through a [`HeadsetView`].
///
/// A model lands on a table before the wearer. A model loads on a thread of
/// its own while the one before it stays in view under a dimmed sky.
pub struct ModelGallery {
    device: Device,

    queue: Queue,

    format: TextureFormat,

    size: [u32; 2],

    /// The fewest pixels a cell spans before a coarser copy draws in its
    /// place, or zero for a gallery whose models keep no copies.
    detail: f64,

    directory: PathBuf,

    /// The models' documents, in the order of their paths.
    paths: Vec<PathBuf>,

    shown: Option<Shown>,

    /// The index of the model the wearer last asked for.
    wanted: usize,

    /// The index of the model on the loading thread, and where its result
    /// lands.
    loading: Option<(usize, Receiver<Loaded>)>,

    /// The index of the model whose load last ended, in view or failed.
    settled: Option<usize>,

    /// Whether the turn stick still leans from its last turn.
    leaning: bool,
}

/// A model in view.
struct Shown {
    view: HeadsetView,

    bounds: TyBoundsF64,

    /// Whether the last switch set the model on the table.
    table: bool,
}

/// What a loading thread sends back: the model's view on its table, its
/// bounds, and the seconds its tiles and then its view took, or why the
/// load failed.
type Loaded = Result<(HeadsetView, TyBoundsF64, [f64; 2])>;

impl ModelGallery {
    /// The format of the depth a pass that draws the gallery clears to
    /// zero.
    pub const DEPTH: TextureFormat = HeadsetView::DEPTH;

    /// Lists the `.voxj` and `.voxjz` documents under `directory` and
    /// starts loading the first for a two-layer target of `format` whose
    /// width and height are `size`. With `first`, the gallery starts at
    /// the first document whose path under `directory` holds that text.
    /// With a `detail` over zero, each model keeps coarser copies of its
    /// objects, and its view takes `detail` as [`HeadsetView::set_detail`]
    /// does. Errors if the directory does not list, holds no document, or
    /// holds none with `first` in its path.
    pub fn new(
        device: &Device,
        queue: &Queue,
        directory: &Path,
        first: Option<&str>,
        format: TextureFormat,
        size: [u32; 2],
        detail: f64,
    ) -> Result<Self> {
        let mut paths = Vec::new();

        list(directory, &mut paths).map_err(|error| Error::Models {
            directory: directory.to_owned(),
            error: error.to_string(),
        })?;

        paths.sort();

        if paths.is_empty() {
            return Err(Error::Models {
                directory: directory.to_owned(),
                error: "it holds no .voxj or .voxjz document".to_owned(),
            });
        }

        let mut gallery = ModelGallery {
            device: device.clone(),
            queue: queue.clone(),
            format,
            size,
            detail,
            directory: directory.to_owned(),
            paths,
            shown: None,
            wanted: 0,
            loading: None,
            settled: None,
            leaning: false,
        };

        if let Some(first) = first {
            gallery.wanted = (0..gallery.paths.len())
                .find(|&index| gallery.name(index).contains(first))
                .ok_or_else(|| Error::Models {
                    directory: directory.to_owned(),
                    error: format!("no document's path holds {first:?}"),
                })?;
        }

        gallery.load();

        Ok(gallery)
    }

    /// How many models the gallery holds, at least one.
    pub fn count(&self) -> usize {
        self.paths.len()
    }

    /// The color a pass that draws the gallery clears its target to: the
    /// sky of the model in view, dimmed while the next model loads.
    pub fn sky(&self) -> Color {
        let Some(shown) = &self.shown else {
            return EMPTY;
        };

        let sky = shown.view.sky();

        if self.loading.is_none() {
            return sky;
        }

        Color {
            r: sky.r * DIM,
            g: sky.g * DIM,
            b: sky.b * DIM,
            a: sky.a,
        }
    }

    /// Readies the next draw for `eyes`, the headset's left eye and then
    /// its right, `seconds` after the last. The gallery first does what
    /// `controls` ask and takes a model that finished loading. Returns the
    /// line that reports such a model, or why it failed to load.
    pub fn prepare(
        &mut self,
        controls: &GalleryControls,
        eyes: &[HeadsetEye; 2],
        seconds: f64,
    ) -> Option<String> {
        let count = i64::try_from(self.paths.len()).expect("a gallery's models number under 2^63");
        let wanted = i64::try_from(self.wanted).expect("an index is under the count");

        self.wanted = usize::try_from((wanted + i64::from(controls.step)).rem_euclid(count))
            .expect("a remainder is under the count");

        let line = self.land();

        if self.loading.is_none() && self.settled != Some(self.wanted) {
            self.load();
        }

        let leaning = &mut self.leaning;

        if let Some(shown) = &mut self.shown {
            shown.view.stand = steered(
                shown.view.stand,
                &shown.bounds,
                &mut shown.table,
                leaning,
                controls,
                eyes,
                seconds,
            );

            shown.view.prepare(&self.queue, eyes);
        }

        line
    }

    /// How many passes draw the gallery, as [`HeadsetView::passes`] counts
    /// them.
    pub fn passes(&self) -> u32 {
        self.shown.as_ref().map_or(1, |shown| shown.view.passes())
    }

    /// Draws the `at`th of the gallery's [`passes`](Self::passes) into
    /// `pass`.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        if let Some(shown) = &self.shown {
            shown.view.draw(pass, at);
        }
    }

    /// Encodes the passes that come before the first of a frame's
    /// [`passes`](Self::passes), as [`HeadsetView::glow`] does.
    pub fn glow(&self, encoder: &mut CommandEncoder) {
        if let Some(shown) = &self.shown {
            shown.view.glow(encoder);
        }
    }

    /// The path of model `index` under the gallery's directory.
    fn name(&self, index: usize) -> String {
        let path = &self.paths[index];

        path.strip_prefix(&self.directory)
            .unwrap_or(path)
            .display()
            .to_string()
    }

    /// Starts a thread that loads the wanted model.
    fn load(&mut self) {
        let (sender, receiver) = mpsc::channel();
        let device = self.device.clone();
        let queue = self.queue.clone();
        let path = self.paths[self.wanted].clone();
        let format = self.format;
        let size = self.size;
        let detail = self.detail;

        thread::spawn(move || {
            let start = Instant::now();

            let glass = BoxView::draws_glass(&device);

            let loaded = ModelTiles::load(&path, glass, detail > 0.0).and_then(|model| {
                let tiled = start.elapsed().as_secs_f64();

                let mut view = HeadsetView::new(
                    &device,
                    &queue,
                    &model.tiles,
                    model.bloom,
                    HeadsetStand::table(&model.bounds),
                    format,
                    size,
                )?;

                view.set_detail(detail);

                Ok((
                    view,
                    model.bounds,
                    [tiled, start.elapsed().as_secs_f64() - tiled],
                ))
            });

            // The gallery may have gone, and then nobody waits.
            let _ = sender.send(loaded);
        });

        self.loading = Some((self.wanted, receiver));
    }

    /// Takes the loading thread's model once it has one, and shows the
    /// model when the wearer still wants it. Returns the line that reports
    /// the model.
    fn land(&mut self) -> Option<String> {
        let (index, receiver) = self.loading.as_ref()?;
        let index = *index;

        let loaded = match receiver.try_recv() {
            Ok(loaded) => loaded,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => Err(Error::Models {
                directory: self.directory.clone(),
                error: "a loading thread stopped".to_owned(),
            }),
        };

        self.loading = None;
        self.settled = Some(index);

        let name = self.name(index);
        let place = format!("{} of {}", index + 1, self.paths.len());

        match loaded {
            Ok((view, bounds, [tiled, viewed])) => {
                if index == self.wanted {
                    self.shown = Some(Shown {
                        view,
                        bounds,
                        table: true,
                    });
                }

                Some(format!(
                    "{place} {name}: loaded in {:.0} ms, its tiles in {:.0} ms",
                    (tiled + viewed) * 1e3,
                    tiled * 1e3
                ))
            }

            // The model before stays in view, and the next step moves past
            // the one that failed.
            Err(error) => Some(format!("{place} {name}: {error}")),
        }
    }
}

/// Appends the `.voxj` and `.voxjz` documents under `directory` to `paths`.
fn list(directory: &Path, paths: &mut Vec<PathBuf>) -> IOResult<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();

        if path.is_dir() {
            list(&path, paths)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "voxj" || extension == "voxjz")
        {
            paths.push(path);
        }
    }

    Ok(())
}

/// `stand` as `controls` leave it after `seconds`, about the head between
/// `eyes`. A switch flips `table` and takes the stand that fits `bounds`.
/// `leaning` holds whether the turn stick still leans from its last turn.
fn steered(
    mut stand: HeadsetStand,
    bounds: &TyBoundsF64,
    table: &mut bool,
    leaning: &mut bool,
    controls: &GalleryControls,
    eyes: &[HeadsetEye; 2],
    seconds: f64,
) -> HeadsetStand {
    let point = |eye: &HeadsetEye| {
        let [x, y, z] = eye.position.map(f64::from);

        TyVector3F64::new(x, y, z)
    };

    let head = (point(&eyes[0]) + point(&eyes[1])) / 2.0;
    let ahead = TyQuaternionF64::from_array(eyes[0].rotation.map(f64::from)) * -TyVector3F64::Z;

    // The way the head faces along the floor.
    let level = TyVector3F64::new(ahead.x, 0.0, ahead.z)
        .try_normalize()
        .unwrap_or(-TyVector3F64::Z);
    let right = TyVector3F64::new(-level.z, 0.0, level.x);

    // A stick's push grows as its square, which keeps slow moves fine.
    let push = |value: f32| {
        let value = f64::from(if value.abs() < DEAD { 0.0 } else { value });

        value * value.abs()
    };

    if controls.resize {
        *table = !*table;

        stand = if *table {
            HeadsetStand::table(bounds)
        } else {
            HeadsetStand::life(bounds)
        };
    }

    let step = right * push(controls.walk[0])
        + level * push(controls.walk[1])
        + TyVector3F64::Y * push(controls.rise);

    stand = stand.moved(step * SPEED * seconds);

    if controls.turn.abs() > LEAN[0] && !*leaning {
        *leaning = true;

        // A lean to the right turns the wearer clockwise from above.
        stand = stand.turned(head, -SNAP.copysign(f64::from(controls.turn)));
    } else if controls.turn.abs() < LEAN[1] {
        *leaning = false;
    }

    let growth = GROWTH.powf(push(controls.grow) * seconds);

    stand.grown(head + ahead * HELD, growth)
}

#[cfg(test)]
mod tests {
    use crate::{GalleryControls, HeadsetEye, HeadsetStand, model_gallery::steered};
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_6};
    use ty_math::{TyBoundsF64, TyQuaternionF64, TyVector3F64};

    /// A stand at twice life size, a quarter turn into its scene.
    fn stand() -> HeadsetStand {
        HeadsetStand {
            origin: TyVector3F64::new(10.0, 0.0, 0.0),
            turn: FRAC_PI_2,
            scale: 2.0,
        }
    }

    /// The eyes of a head at `head` that has turned `turn` radians to its
    /// left.
    fn eyes(head: [f32; 3], turn: f64) -> [HeadsetEye; 2] {
        let rotation = TyQuaternionF64::from_rotation_y(turn);
        let right = (rotation * TyVector3F64::X * 0.03).as_vec3().to_array();

        [-1.0, 1.0].map(|side: f32| HeadsetEye {
            position: [0, 1, 2].map(|axis| head[axis] + side * right[axis]),
            rotation: rotation.to_array().map(|value| value as f32),
            tangents: [-1.0, 1.0, 1.0, -1.0],
        })
    }

    /// `stand()` after a second of `controls` under a head at `head` turned
    /// `turn` to its left.
    fn after(controls: GalleryControls, head: [f32; 3], turn: f64) -> HeadsetStand {
        steered(
            stand(),
            &bounds(),
            &mut true,
            &mut false,
            &controls,
            &eyes(head, turn),
            1.0,
        )
    }

    fn bounds() -> TyBoundsF64 {
        TyBoundsF64::new(TyVector3F64::ZERO, TyVector3F64::ONE)
    }

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-6
    }

    #[test]
    fn the_sticks_move_the_wearer_the_way_the_head_faces() {
        // A full stick moves 1.5 m of the headset's a second, 3 m of this
        // scene's. The stand's quarter turn carries the headset's -Z to
        // the scene's -X and its +X to the scene's -Z.
        let ahead = after(
            GalleryControls {
                walk: [0.0, 1.0],
                ..GalleryControls::default()
            },
            [0.0; 3],
            0.0,
        );
        assert!(close(ahead.origin, TyVector3F64::new(7.0, 0.0, 0.0)));

        let right = after(
            GalleryControls {
                walk: [1.0, 0.0],
                ..GalleryControls::default()
            },
            [0.0; 3],
            0.0,
        );
        assert!(close(right.origin, TyVector3F64::new(10.0, 0.0, -3.0)));

        // A head turned a quarter to its left faces the headset's -X, the
        // scene's +Z.
        let turned = after(
            GalleryControls {
                walk: [0.0, 1.0],
                ..GalleryControls::default()
            },
            [0.0; 3],
            FRAC_PI_2,
        );
        assert!(close(turned.origin, TyVector3F64::new(10.0, 0.0, 3.0)));

        let up = after(
            GalleryControls {
                rise: -1.0,
                ..GalleryControls::default()
            },
            [0.0; 3],
            0.0,
        );
        assert!(close(up.origin, TyVector3F64::new(10.0, -3.0, 0.0)));

        // A stick near its center rests.
        let resting = after(
            GalleryControls {
                walk: [0.1, -0.1],
                rise: 0.1,
                grow: 0.1,
                ..GalleryControls::default()
            },
            [0.0; 3],
            0.0,
        );
        assert_eq!(resting, stand());
    }

    #[test]
    fn a_lean_turns_the_wearer_once_about_the_head() {
        let head = [0.5, 0.0, -1.0];
        let lean = GalleryControls {
            turn: 1.0,
            ..GalleryControls::default()
        };

        let mut leaning = false;
        let mut step = |stand, controls: &GalleryControls| {
            steered(
                stand,
                &bounds(),
                &mut true,
                &mut leaning,
                controls,
                &eyes(head, 0.0),
                1.0,
            )
        };

        // A lean to the right turns the wearer clockwise from above.
        let once = step(stand(), &lean);
        assert!((once.turn - (FRAC_PI_2 - FRAC_PI_6)).abs() < 1e-12);

        let head = TyVector3F64::new(0.5, 0.0, -1.0);
        assert!(close(once.to_scene(head), stand().to_scene(head)));

        // The lean turns no more until the stick has come back.
        assert_eq!(step(once, &lean), once);

        let back = step(once, &GalleryControls::default());
        let left = step(
            back,
            &GalleryControls {
                turn: -1.0,
                ..GalleryControls::default()
            },
        );
        assert!((left.turn - FRAC_PI_2).abs() < 1e-12);
    }

    #[test]
    fn a_trigger_grows_the_model_around_the_point_ahead() {
        let grown = after(
            GalleryControls {
                grow: 1.0,
                ..GalleryControls::default()
            },
            [0.0, 0.2, 0.0],
            0.0,
        );
        assert!((grown.scale - 0.5).abs() < 1e-12);

        let held = TyVector3F64::new(0.0, 0.2, -1.0);
        assert!(close(grown.to_scene(held), stand().to_scene(held)));

        let shrunk = after(
            GalleryControls {
                grow: -1.0,
                ..GalleryControls::default()
            },
            [0.0, 0.2, 0.0],
            0.0,
        );
        assert!((shrunk.scale - 8.0).abs() < 1e-12);
    }

    #[test]
    fn a_switch_goes_from_the_table_to_life_size_and_back() {
        let switch = GalleryControls {
            resize: true,
            ..GalleryControls::default()
        };

        let mut table = true;
        let mut step = |stand| {
            steered(
                stand,
                &bounds(),
                &mut table,
                &mut false,
                &switch,
                &eyes([0.0; 3], 0.0),
                1.0,
            )
        };

        let life = step(stand());
        assert_eq!(life, HeadsetStand::life(&bounds()));
        assert_eq!(step(life), HeadsetStand::table(&bounds()));
    }
}
