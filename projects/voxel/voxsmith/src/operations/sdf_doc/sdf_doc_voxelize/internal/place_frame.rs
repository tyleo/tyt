use crate::operations::sdf_doc::SdfPlace;
use ty_math::TyPoseF64;

/// How a place's grid relates to the node that holds it. A kept place writes
/// its own node, and every other place flattens into the nearest kept place
/// above it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaceFrame {
    /// Whether the place writes its own node.
    pub kept: bool,

    /// The map from the place's offset frame onto the offset frame of the
    /// nearest kept place above it, which turns about each pivot on the way. A
    /// root part's map is the identity.
    pub to_kept: TyPoseF64,
}

impl PlaceFrame {
    /// The frame of each of `places`, which list each place before its
    /// children. Unposed, every place is kept. Posed, only the root parts and
    /// the joints are.
    pub fn of_places(places: &[SdfPlace], posed: bool) -> Vec<Self> {
        let mut frames: Vec<Self> = Vec::with_capacity(places.len());

        for place in places {
            let frame = match place.parent {
                Some(parent) => {
                    let base = frames[parent].sampling();
                    let rotation = base.rotation * place.rotation;

                    // The place turns `x` about its pivot to
                    // `place.rotation * (x - pivot) + pivot`, and the base
                    // frame carries the result.

                    Self {
                        kept: !posed || place.joint,
                        to_kept: TyPoseF64::new(
                            base.position
                                + base.rotation * (place.pivot - place.rotation * place.pivot),
                            rotation,
                        ),
                    }
                }

                None => Self {
                    kept: true,
                    to_kept: TyPoseF64::IDENTITY,
                },
            };

            frames.push(frame);
        }

        frames
    }

    /// The map the place's shapes sample through before its offset: the
    /// identity for a kept place, which samples in its own frame.
    pub fn sampling(&self) -> TyPoseF64 {
        if self.kept {
            TyPoseF64::IDENTITY
        } else {
            self.to_kept
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{PlaceFrame, SdfPlace};
    use branded_id::U32Id;
    use std::f64::consts::FRAC_PI_2;
    use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3F64};

    fn place(
        parent: Option<usize>,
        pivot: TyVector3F64,
        rotation: TyQuaternionF64,
        joint: bool,
    ) -> SdfPlace {
        SdfPlace {
            node_id: U32Id::from_u32(0),
            parent,
            path: Vec::new(),
            pivot,
            offset: TyVector3F64::ZERO,
            rotation,
            joint,
            grid_indices: Vec::new(),
        }
    }

    /// A root holding an arm turned about z at the origin, which holds a hand
    /// joint turned about y at `[2, 0, 0]`.
    fn arm_and_hand() -> Vec<SdfPlace> {
        vec![
            place(None, TyVector3F64::ZERO, TyQuaternionF64::IDENTITY, false),
            place(
                Some(0),
                TyVector3F64::ZERO,
                TyQuaternionF64::from_rotation_z(FRAC_PI_2),
                false,
            ),
            place(
                Some(1),
                TyVector3F64::new(2.0, 0.0, 0.0),
                TyQuaternionF64::from_rotation_y(FRAC_PI_2),
                true,
            ),
        ]
    }

    #[test]
    fn posed_a_joint_keeps_its_node_through_the_turns_above_it() {
        let frames = PlaceFrame::of_places(&arm_and_hand(), true);
        assert_eq!(
            frames.iter().map(|frame| frame.kept).collect::<Vec<_>>(),
            [true, false, true]
        );

        // The arm's turn carries the hand's pivot onto +y, and the hand's turn
        // followed by the arm's carries +x onto -z.
        let hand = frames[2];
        assert_eq!(hand.sampling(), TyPoseF64::IDENTITY);
        assert!(
            (hand.to_kept.position + hand.to_kept.rotation * TyVector3F64::new(2.0, 0.0, 0.0))
                .abs_diff_eq(TyVector3F64::new(0.0, 2.0, 0.0), 1e-12)
        );
        assert!((hand.to_kept.rotation * TyVector3F64::X).abs_diff_eq(-TyVector3F64::Z, 1e-12));
    }

    #[test]
    fn unposed_every_place_keeps_its_node() {
        let frames = PlaceFrame::of_places(&arm_and_hand(), false);
        assert!(frames.iter().all(|frame| frame.kept));
    }
}
