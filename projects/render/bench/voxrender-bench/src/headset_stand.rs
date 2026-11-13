use crate::HeadsetEye;
use std::f64::consts::{FRAC_PI_3, TAU};
use ty_math::{TyBoundsF64, TyQuaternionF64, TyVector3F64};
use voxrender::{RenderView, fit_scale};

/// The meters across a model's bounding sphere on [`HeadsetStand::table`].
const TABLE: f64 = 1.0;

/// The meters from the tracking space's origin ahead to a table model's
/// center, and down to it.
const AHEAD: f64 = 1.1;
const BELOW: f64 = 0.25;

/// The meters from a life-size model's floor up to the tracking space's
/// origin at a standing wearer's eyes.
const EYES: f64 = 1.6;

/// The meters a life-size wearer stands back from a model's front.
const BACK: f64 = 1.5;

/// The least and the most scene meters a meter of the tracking space spans.
const SCALES: [f64; 2] = [1.0 / 1024.0, 4096.0];

/// The meters a walk goes ahead before it turns back.
const WALK: f64 = 60.0;

/// The steps a second of a walk, and the meters a step lifts the head by
/// and shifts it to a side by.
const STEPS: f64 = 2.0;
const BOB: f64 = 0.03;
const SWAY: f64 = 0.02;

/// The radians a walker's head turns to each side, and the seconds from
/// one look to a side to the next look there.
const LOOK: f64 = FRAC_PI_3;
const LOOKS: f64 = 4.0;

/// Where a headset's tracking space sits in a scene. The space's -Z is
/// ahead of the wearer and its +Y is up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeadsetStand {
    /// The tracking space's origin in the scene.
    pub origin: TyVector3F64,

    /// The radians the tracking space turns about +Y into the scene.
    pub turn: f64,

    /// The scene meters a meter of the tracking space spans.
    pub scale: f64,
}

impl HeadsetStand {
    /// The stand at `view`'s position at life size. It faces the way the view
    /// looks and stays level.
    pub fn at_view(view: &RenderView) -> Self {
        let forward = view.pose.rotation * -TyVector3F64::Z;

        HeadsetStand {
            origin: view.pose.position,
            turn: (-forward.x).atan2(-forward.z),
            scale: 1.0,
        }
    }

    /// The stand that shows the model within `bounds` as on a table: shrunk
    /// or grown to [`TABLE`] meters across, ahead of the wearer and below
    /// their eyes, with its +Z side facing them.
    pub fn table(bounds: &TyBoundsF64) -> Self {
        let scale = fit_scale(bounds) / TABLE;

        HeadsetStand {
            origin: bounds.center + TyVector3F64::new(0.0, BELOW, AHEAD) * scale,
            turn: 0.0,
            scale,
        }
    }

    /// The stand that shows the model within `bounds` at life size: the
    /// wearer stands on its floor, back from its +Z side, and faces it.
    pub fn life(bounds: &TyBoundsF64) -> Self {
        HeadsetStand {
            origin: TyVector3F64::new(
                bounds.center.x,
                bounds.min().y + EYES,
                bounds.max().z + BACK,
            ),
            turn: 0.0,
            scale: 1.0,
        }
    }

    /// The stand `seconds` into a walk from this one at `speed` meters of
    /// the tracking space a second. The walk goes [`WALK`] meters ahead and
    /// back. The head between `eyes` rises and sways with each step and
    /// looks from side to side.
    pub fn walked(self, eyes: &[HeadsetEye; 2], seconds: f64, speed: f64) -> Self {
        let along = (speed * seconds) % (2.0 * WALK);
        let ahead = WALK - (along - WALK).abs();
        let bob = BOB * (TAU * STEPS * seconds).sin();
        let sway = SWAY * (TAU * STEPS / 2.0 * seconds).sin();
        let look = LOOK * (TAU * seconds / LOOKS).sin();

        let [x, y, z] =
            [0, 1, 2].map(|axis| f64::from(eyes[0].position[axis] + eyes[1].position[axis]) / 2.0);

        self.moved(TyVector3F64::new(sway, bob, -ahead))
            .turned(TyVector3F64::new(x, y, z), look)
    }

    /// The rotation from the tracking space into the scene.
    pub fn rotation(&self) -> TyQuaternionF64 {
        TyQuaternionF64::from_rotation_y(self.turn)
    }

    /// The scene's point at `point` of the tracking space.
    pub fn to_scene(&self, point: TyVector3F64) -> TyVector3F64 {
        self.origin + self.rotation() * (point * self.scale)
    }

    /// The stand moved by `step`, in meters of the tracking space.
    pub fn moved(self, step: TyVector3F64) -> Self {
        HeadsetStand {
            origin: self.to_scene(step),
            ..self
        }
    }

    /// The stand turned by `angle` radians about the vertical through
    /// `pivot`, a point of the tracking space that keeps its place in the
    /// scene.
    pub fn turned(self, pivot: TyVector3F64, angle: f64) -> Self {
        self.around(pivot, self.turn + angle, self.scale)
    }

    /// The stand with the scene `factor` times as large around `pivot`, a
    /// point of the tracking space that keeps its place in the scene. The
    /// scale stays within [`SCALES`].
    pub fn grown(self, pivot: TyVector3F64, factor: f64) -> Self {
        let scale = (self.scale / factor).clamp(SCALES[0], SCALES[1]);

        self.around(pivot, self.turn, scale)
    }

    /// The stand at `turn` and `scale` that keeps `pivot`'s place in the
    /// scene.
    fn around(self, pivot: TyVector3F64, turn: f64, scale: f64) -> Self {
        let held = self.to_scene(pivot);

        let stand = HeadsetStand {
            origin: TyVector3F64::ZERO,
            turn,
            scale,
        };

        HeadsetStand {
            origin: held - stand.to_scene(pivot),
            ..stand
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{HeadsetEye, HeadsetStand};
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};
    use ty_math::{TyBoundsF64, TyVector3F64};
    use voxrender::fit_scale;

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    fn stand() -> HeadsetStand {
        HeadsetStand {
            origin: TyVector3F64::new(1.0, 2.0, 3.0),
            turn: FRAC_PI_2,
            scale: 4.0,
        }
    }

    #[test]
    fn a_point_scales_turns_and_then_shifts_into_the_scene() {
        // A quarter turn about +Y carries the space's -Z to the scene's -X.
        assert!(close(
            stand().to_scene(TyVector3F64::new(0.0, 1.0, -1.0)),
            TyVector3F64::new(-3.0, 6.0, 3.0)
        ));
    }

    #[test]
    fn a_move_steps_in_the_tracking_spaces_meters() {
        let moved = stand().moved(TyVector3F64::new(0.0, 0.0, -1.0));

        assert!(close(moved.origin, TyVector3F64::new(-3.0, 2.0, 3.0)));
        assert_eq!((moved.turn, moved.scale), (FRAC_PI_2, 4.0));
    }

    #[test]
    fn a_turn_and_a_growth_keep_their_pivots_place() {
        let pivot = TyVector3F64::new(0.5, -0.25, -1.0);
        let held = stand().to_scene(pivot);

        let turned = stand().turned(pivot, 0.5);
        assert!(close(turned.to_scene(pivot), held));
        assert_eq!((turned.turn, turned.scale), (FRAC_PI_2 + 0.5, 4.0));

        let grown = stand().grown(pivot, 8.0);
        assert!(close(grown.to_scene(pivot), held));
        assert_eq!((grown.turn, grown.scale), (FRAC_PI_2, 0.5));

        // A growth past the least scale stops there.
        assert_eq!(stand().grown(pivot, 1e9).scale, 1.0 / 1024.0);
    }

    #[test]
    fn a_walk_goes_ahead_and_comes_back() {
        let eyes = [-0.03, 0.03].map(|x| HeadsetEye {
            position: [x, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            tangents: [-1.0, 1.0, 1.0, -1.0],
        });

        let from = HeadsetStand {
            origin: TyVector3F64::ZERO,
            turn: 0.0,
            scale: 1.0,
        };

        // Whole seconds land between steps and between looks to a side.
        let walked = from.walked(&eyes, 10.0, 2.0);
        assert!(close(walked.origin, TyVector3F64::new(0.0, 0.0, -20.0)));
        assert!(walked.turn.abs() < 1e-9);

        // 50 s at 2 m a second is 60 m ahead and 40 m back.
        assert!(close(
            from.walked(&eyes, 50.0, 2.0).origin,
            TyVector3F64::new(0.0, 0.0, -20.0)
        ));

        // A look to a side turns the stand about the head.
        let looking = from.walked(&eyes, 1.0, 2.0);
        assert!((looking.turn - FRAC_PI_3).abs() < 1e-9);
        assert!(close(looking.origin, TyVector3F64::new(0.0, 0.0, -2.0)));
    }

    #[test]
    fn a_table_sets_the_model_ahead_and_a_life_stands_before_it() {
        let bounds = TyBoundsF64::new(
            TyVector3F64::new(10.0, 5.0, -20.0),
            TyVector3F64::new(3.0, 4.0, 12.0),
        );

        let table = HeadsetStand::table(&bounds);
        assert!((table.scale - fit_scale(&bounds)).abs() < 1e-9);
        assert!(close(
            table.to_scene(TyVector3F64::new(0.0, -0.25, -1.1)),
            bounds.center
        ));

        let life = HeadsetStand::life(&bounds);
        assert_eq!(life.scale, 1.0);
        assert!(close(life.origin, TyVector3F64::new(10.0, 2.6, -6.5)));
    }
}
