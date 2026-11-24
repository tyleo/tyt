use std::{
    collections::HashSet,
    f64::consts::{FRAC_PI_2, FRAC_PI_3},
};
use ty_math::{TyBoundsF64, TyQuaternionF64, TyVector3F64};
use voxrender_bench::{GalleryControls, HeadsetEye, HeadsetStand};
use winit::keyboard::KeyCode;

/// The radians a window's image spans from its top to its bottom.
const SPAN: f64 = FRAC_PI_3;

/// The radians the head turns as the pointer drags across the window's
/// height.
const TURN: f64 = 2.0;

/// The most radians the head tips up or down.
const TIP: f64 = FRAC_PI_2 * 89.0 / 90.0;

/// A keyboard and a mouse as a gallery's controls and as the head its view
/// looks from:
///
/// 1. W, A, S, and D walk, as the left and right arrows do to each side. E
///    rises, and Q sinks
/// 2. A drag turns the head
/// 3. The up arrow grows the model around what the window's middle shows.
///    The down arrow shrinks the model
/// 4. N steps to the next model, and P to the one before
/// 5. Tab switches the model between the table and life size
pub struct GalleryKeys {
    /// The keys held down.
    down: HashSet<KeyCode>,

    /// The models the presses since the last read step by.
    step: i32,

    /// Whether a press since the last read switches the model's size.
    resize: bool,

    /// The radians the head has turned to its left.
    turn: f64,

    /// The radians the head has tipped up.
    tip: f64,

    /// The meters ahead of the head to what the window's middle last
    /// showed.
    held: f64,
}

impl GalleryKeys {
    /// Keys with none held, under a head that faces the center of a model
    /// on its table.
    pub fn new() -> Self {
        let bounds = TyBoundsF64::new(TyVector3F64::ZERO, TyVector3F64::ONE);
        let stand = HeadsetStand::table(&bounds);
        let center = (bounds.center - stand.origin) / stand.scale;

        GalleryKeys {
            down: HashSet::new(),
            step: 0,
            resize: false,
            turn: 0.0,
            tip: center.y.atan2(-center.z),
            held: center.length(),
        }
    }

    /// Turns the head back to where it started.
    pub fn restart(&mut self) {
        let start = Self::new();

        self.turn = start.turn;
        self.tip = start.tip;
        self.held = start.held;
    }

    /// Takes the meters ahead of the head to what the window's middle shows.
    pub fn hold(&mut self, meters: f64) {
        self.held = meters;
    }

    /// Takes the key `code` going down or coming up. `repeat` marks a press
    /// the keyboard sends while the key stays down.
    pub fn key(&mut self, code: KeyCode, down: bool, repeat: bool) {
        if !down {
            self.down.remove(&code);

            return;
        }

        self.down.insert(code);

        if repeat {
            return;
        }

        match code {
            KeyCode::KeyN => self.step += 1,
            KeyCode::KeyP => self.step -= 1,
            KeyCode::Tab => self.resize = true,
            _ => {}
        }
    }

    /// Lets go of every key.
    pub fn release(&mut self) {
        self.down.clear();
    }

    /// Turns the head for a pointer drag `across` to the right and `down`,
    /// each as a part of the window's height.
    pub fn drag(&mut self, across: f64, down: f64) {
        self.turn -= across * TURN;
        self.tip = (self.tip - down * TURN).clamp(-TIP, TIP);
    }

    /// What the keys ask of the next frame. The steps and the switch then
    /// start over.
    pub fn read(&mut self) -> GalleryControls {
        let any = |codes: &[KeyCode]| codes.iter().any(|code| self.down.contains(code));
        let axis = |up: &[KeyCode], down: &[KeyCode]| {
            f32::from(u8::from(any(up))) - f32::from(u8::from(any(down)))
        };

        let controls = GalleryControls {
            step: self.step,
            resize: self.resize,
            walk: [
                axis(
                    &[KeyCode::KeyD, KeyCode::ArrowRight],
                    &[KeyCode::KeyA, KeyCode::ArrowLeft],
                ),
                axis(&[KeyCode::KeyW], &[KeyCode::KeyS]),
            ],
            turn: 0.0,
            rise: axis(&[KeyCode::KeyE], &[KeyCode::KeyQ]),
            grow: axis(&[KeyCode::ArrowUp], &[KeyCode::ArrowDown]),
            held: self.held as f32,
        };

        self.step = 0;
        self.resize = false;

        controls
    }

    /// The head as both of a headset's eyes, for a window `aspect` times as
    /// wide as it is tall. The head sits at the tracking space's origin.
    pub fn eyes(&self, aspect: f64) -> [HeadsetEye; 2] {
        let rotation = TyQuaternionF64::from_rotation_y(self.turn)
            * TyQuaternionF64::from_rotation_x(self.tip);
        let up = (SPAN / 2.0).tan();
        let right = up * aspect;

        [HeadsetEye {
            position: [0.0; 3],
            rotation: rotation.to_array().map(|value| value as f32),
            tangents: [-right, right, up, -up].map(|value| value as f32),
        }; 2]
    }
}

#[cfg(test)]
mod tests {
    use crate::GalleryKeys;
    use ty_math::{TyQuaternionF64, TyVector3F64};
    use winit::keyboard::KeyCode;

    /// The way the head of `keys` faces.
    fn ahead(keys: &GalleryKeys) -> TyVector3F64 {
        let [eye, _] = keys.eyes(1.0);

        TyQuaternionF64::from_array(eye.rotation.map(f64::from)) * -TyVector3F64::Z
    }

    #[test]
    fn a_press_steps_once_and_a_held_key_walks() {
        let mut keys = GalleryKeys::new();

        keys.key(KeyCode::KeyN, true, false);
        keys.key(KeyCode::KeyN, true, true);
        keys.key(KeyCode::Tab, true, false);
        keys.key(KeyCode::KeyW, true, false);
        keys.key(KeyCode::KeyA, true, false);

        let first = keys.read();

        assert_eq!(first.step, 1);
        assert!(first.resize);
        assert_eq!(first.walk, [-1.0, 1.0]);

        keys.key(KeyCode::KeyA, false, false);

        let second = keys.read();

        assert_eq!(second.step, 0);
        assert!(!second.resize);
        assert_eq!(second.walk, [0.0, 1.0]);

        keys.release();

        assert_eq!(keys.read().walk, [0.0, 0.0]);

        // An arrow walks to its side as A and D do.
        keys.key(KeyCode::ArrowRight, true, false);

        let third = keys.read();

        assert_eq!(third.walk, [1.0, 0.0]);
        assert_eq!(third.step, 0);
    }

    #[test]
    fn a_growing_model_holds_what_the_windows_middle_shows() {
        let mut keys = GalleryKeys::new();

        // A table model's center sits 1.1 m ahead and 0.25 m down.
        assert!((keys.read().held - 1.128).abs() < 1e-3);

        keys.hold(40.0);
        assert_eq!(keys.read().held, 40.0);
    }

    #[test]
    fn a_drag_turns_the_head_the_way_the_pointer_moves() {
        let mut keys = GalleryKeys::new();

        // The head starts level across and tipped down to the table.
        let start = ahead(&keys);

        assert!(start.x.abs() < 1e-6 && start.y < 0.0 && start.z < 0.0);

        keys.drag(0.25, 0.0);
        assert!(ahead(&keys).x > 0.0);

        keys.drag(0.0, -0.5);
        assert!(ahead(&keys).y > 0.0);

        // The head tips no further than straight up.
        keys.drag(0.0, -10.0);
        assert!(ahead(&keys).y < 1.0);

        keys.restart();
        assert!((ahead(&keys) - start).length() < 1e-12);
    }

    #[test]
    fn the_eyes_take_the_windows_shape() {
        let [eye, _] = GalleryKeys::new().eyes(2.0);
        let [left, right, up, down] = eye.tangents;

        assert_eq!([left, down], [-right, -up]);
        assert!((right / up - 2.0).abs() < 1e-6);
    }
}
