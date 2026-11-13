use crate::operations::sdf_doc::{
    Bounds3d, SdfEvaluation, SdfShapes, Shape3dField, ShapeSource, UnionTree, box_distance,
    smooth_union_distance,
};
use branded_id::{IdVec, IteratorExt, U32Id};
use sdfcore::{BSdfShape2d, BSdfShape3d};
use std::sync::OnceLock;
use ty_math::{TyPoseF64, TyQuaternionF64, TyVector2F64, TyVector3F64};

/// How far a turned box's corner can sit from a cell corner, in cells, and
/// still count as on it. A quarter turn's rounding then adds no empty layer.
const TURNED_CORNER_TOLERANCE: f64 = 1e-9;

/// A model's shapes posed onto the lattice of cells `voxel_size` across. Each
/// 3D shape's distance takes the larger of its formula's value and the signed
/// distance to the lattice box around its posed box, rounded out to the
/// lattice's cell corners.
///
/// That floor lets unions and smooth unions skip any child whose box lies too
/// far away to change the result.
#[derive(Clone, Debug)]
pub struct LatticeShapes<'a> {
    shapes: &'a SdfShapes,

    pose: TyPoseF64,

    /// Each box's corners as whole numbers of cells in the lattice.
    cells: IdVec<BSdfShape3d, Option<Bounds3d>>,

    /// The center and half extents of each box on the lattice's axes, moved
    /// back by the pose's position alone.
    boxes: IdVec<BSdfShape3d, Option<(TyVector3F64, TyVector3F64)>>,

    /// The position in `trees` of each union whose children all hold finite
    /// boxes.
    tree_slots: IdVec<BSdfShape3d, Option<u32>>,

    /// The tree over each such union's children. The union's first
    /// evaluation builds the tree.
    trees: Vec<OnceLock<UnionTree>>,
}

impl<'a> LatticeShapes<'a> {
    /// `shapes` posed by `pose` onto the lattice of cells `voxel_size` across.
    /// `pose` takes a shape's point to its lattice point in meters.
    pub fn new(shapes: &'a SdfShapes, voxel_size: f64, pose: TyPoseF64) -> Self {
        let cells: IdVec<BSdfShape3d, Option<Bounds3d>> = shapes
            .all_bounds3d()
            .iter()
            .map(|bounds| {
                bounds.map(|bounds| {
                    // Bit `axis` of `corner` picks that axis's min or max, so
                    // the 8 values of `corner` walk the box's 8 corners.
                    let corners = (0..8).map(|corner| {
                        let pick = |axis: usize, min: f64, max: f64| {
                            if corner >> axis & 1 == 0 { min } else { max }
                        };

                        pose.position
                            + pose.rotation
                                * TyVector3F64::new(
                                    pick(0, bounds.min.x, bounds.max.x),
                                    pick(1, bounds.min.y, bounds.max.y),
                                    pick(2, bounds.min.z, bounds.max.z),
                                )
                    });

                    let (min, max) = corners.fold(
                        (TyVector3F64::INFINITY, TyVector3F64::NEG_INFINITY),
                        |(min, max), corner| (min.min(corner), max.max(corner)),
                    );

                    let snap = |corner: TyVector3F64| {
                        if pose.rotation == TyQuaternionF64::IDENTITY {
                            return corner;
                        }

                        let whole = corner.round();
                        let near = (corner - whole)
                            .abs()
                            .cmple(TyVector3F64::splat(TURNED_CORNER_TOLERANCE));
                        TyVector3F64::select(near, whole, corner)
                    };

                    Bounds3d {
                        min: snap(min / voxel_size).floor(),
                        max: snap(max / voxel_size).ceil(),
                    }
                })
            })
            .collect();

        let boxes: IdVec<BSdfShape3d, Option<(TyVector3F64, TyVector3F64)>> = cells
            .iter()
            .map(|cells| {
                cells.map(|cells| {
                    let min = cells.min * voxel_size - pose.position;
                    let max = cells.max * voxel_size - pose.position;
                    ((min + max) / 2.0, (max - min) / 2.0)
                })
            })
            .collect();

        let mut tree_count = 0;
        let tree_slots = cells
            .iter()
            .enumerate_ids()
            .map(|(shape3d_id, _)| match shapes.field3d(shape3d_id) {
                Shape3dField::Union { shape_ids }
                    if shape_ids.iter().all(|shape_id| {
                        boxes[shape_id.to_usize_id()].is_some_and(|(center, half_extents)| {
                            center.is_finite() && half_extents.is_finite()
                        })
                    }) =>
                {
                    tree_count += 1;
                    Some(tree_count - 1)
                }

                _ => None,
            })
            .collect();

        Self {
            shapes,
            pose,
            cells,
            boxes,
            tree_slots,
            trees: (0..tree_count).map(|_| OnceLock::new()).collect(),
        }
    }

    /// The rounded box of the shape at `shape3d_id`, with its corners as whole
    /// numbers of cells in the lattice, or `None` for a shape that reaches
    /// without end.
    pub fn cells(&self, shape3d_id: U32Id<BSdfShape3d>) -> Option<Bounds3d> {
        self.cells[shape3d_id.to_usize_id()]
    }

    /// The shape at `shape3d_id` evaluated at `point`. `turned` holds `point`
    /// turned onto the lattice's axes.
    fn evaluate_turned(
        &self,
        shape3d_id: U32Id<BSdfShape3d>,
        point: TyVector3F64,
        turned: TyVector3F64,
    ) -> SdfEvaluation {
        match self.boxes[shape3d_id.to_usize_id()] {
            Some(lattice_box) => {
                self.clamped(shape3d_id, point, turned, to_box(turned, lattice_box))
            }

            None => self.formula(shape3d_id, point, turned),
        }
    }

    /// The formula of the shape at `shape3d_id` evaluated at `point`, with its
    /// distance raised to at least `to_box`, the signed distance from `turned`
    /// to the shape's box.
    fn clamped(
        &self,
        shape3d_id: U32Id<BSdfShape3d>,
        point: TyVector3F64,
        turned: TyVector3F64,
        to_box: f64,
    ) -> SdfEvaluation {
        let evaluation = self.formula(shape3d_id, point, turned);

        SdfEvaluation {
            distance: evaluation.distance.max(to_box),
            ..evaluation
        }
    }

    /// The formula of the shape at `shape3d_id` evaluated at `point`.
    fn formula(
        &self,
        shape3d_id: U32Id<BSdfShape3d>,
        point: TyVector3F64,
        turned: TyVector3F64,
    ) -> SdfEvaluation {
        let field = self.shapes.field3d(shape3d_id);

        match field {
            Shape3dField::Union { shape_ids } => match self.tree(shape3d_id, shape_ids) {
                // The tree needs distances that aren't NaN. A box distance
                // can read NaN only at a non-finite point, so the plain fold
                // handles that point.
                Some(tree) if point.is_finite() => tree.nearest(turned, |index, to_box| {
                    self.clamped(shape_ids[index], point, turned, to_box)
                }),

                _ => field.evaluate(self, point),
            },

            Shape3dField::SmoothUnion { radius, shape_ids } => {
                self.smooth_union(*radius, shape_ids, point, turned)
            }

            _ => field.evaluate(self, point),
        }
    }

    /// The tree over the children `shape_ids` of the union at `shape3d_id`,
    /// or `None` when a child lacks a finite box.
    fn tree(
        &self,
        shape3d_id: U32Id<BSdfShape3d>,
        shape_ids: &[U32Id<BSdfShape3d>],
    ) -> Option<&UnionTree> {
        let slot = self.tree_slots[shape3d_id.to_usize_id()]?;

        Some(self.trees[slot as usize].get_or_init(|| {
            let boxes: Vec<(TyVector3F64, TyVector3F64)> = shape_ids
                .iter()
                .map(|shape_id| {
                    self.boxes[shape_id.to_usize_id()].expect("a tree's children hold boxes")
                })
                .collect();

            UnionTree::new(&boxes)
        }))
    }

    /// The smooth union of `shape_ids` with `radius` evaluated at `point`,
    /// folded in list order as [`Shape3dField`] folds it.
    ///
    /// A child changes neither the blend nor the nearest child when its box
    /// lies at least `radius` past the blend so far and no nearer than the
    /// nearest child so far. The fold skips such a child.
    fn smooth_union(
        &self,
        radius: f64,
        shape_ids: &[U32Id<BSdfShape3d>],
        point: TyVector3F64,
        turned: TyVector3F64,
    ) -> SdfEvaluation {
        let (first_id, rest_ids) = shape_ids
            .split_first()
            .expect("a smooth union holds a shape");
        let first = self.evaluate_turned(*first_id, point, turned);
        let mut distance = first.distance;
        let mut nearest = first;

        for shape_id in rest_ids {
            let evaluation = match self.boxes[shape_id.to_usize_id()] {
                Some(lattice_box) => {
                    let to_box = to_box(turned, lattice_box);

                    if to_box - distance >= radius && to_box >= nearest.distance {
                        continue;
                    }

                    self.clamped(*shape_id, point, turned, to_box)
                }

                None => self.formula(*shape_id, point, turned),
            };

            distance = smooth_union_distance(distance, evaluation.distance, radius);

            if evaluation.distance < nearest.distance {
                nearest = evaluation;
            }
        }

        SdfEvaluation {
            distance,
            ..nearest
        }
    }
}

impl ShapeSource for LatticeShapes<'_> {
    fn evaluate(&self, shape3d_id: U32Id<BSdfShape3d>, point: TyVector3F64) -> SdfEvaluation {
        self.evaluate_turned(shape3d_id, point, self.pose.rotation * point)
    }

    fn distance2d(&self, shape2d_id: U32Id<BSdfShape2d>, point: TyVector2F64) -> f64 {
        self.shapes.field2d(shape2d_id).distance(self, point)
    }
}

/// The signed distance from `turned` to the box whose center and half
/// extents `lattice_box` holds.
fn to_box(turned: TyVector3F64, (center, half_extents): (TyVector3F64, TyVector3F64)) -> f64 {
    box_distance(turned - center, half_extents)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        Bounds3d, LatticeShapes, SdfEvaluation, ShapeSource, box_distance, grid3d, shapes_of,
        smooth_union_distance,
    };
    use branded_id::U32Id;
    use sdfcore::{BSdfShape3d, SdfShape3d};
    use std::f64::consts::FRAC_PI_2;
    use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3F64};

    fn id(index: u32) -> U32Id<BSdfShape3d> {
        U32Id::from_u32(index)
    }

    /// Whether `a` and `b` read the same bits.
    fn same(a: SdfEvaluation, b: SdfEvaluation) -> bool {
        a.distance.to_bits() == b.distance.to_bits()
            && a.frame_position.to_array().map(f64::to_bits)
                == b.frame_position.to_array().map(f64::to_bits)
    }

    #[test]
    fn each_box_rounds_out_to_the_cells_and_clamps_the_distance() {
        let shapes = shapes_of(
            Vec::new(),
            vec![SdfShape3d::Ellipsoid {
                center: TyVector3F64::ZERO,
                radii: TyVector3F64::new(1.0, 0.1, 0.3),
            }],
        );
        let shift = TyVector3F64::new(0.1, 0.0, -0.1);
        let lattice = LatticeShapes::new(
            &shapes,
            0.25,
            TyPoseF64::new(shift, TyQuaternionF64::IDENTITY),
        );

        let cells = lattice.cells(U32Id::from_u32(0)).unwrap();
        assert_eq!(
            cells,
            Bounds3d {
                min: TyVector3F64::new(-4.0, -1.0, -2.0),
                max: TyVector3F64::new(5.0, 1.0, 1.0),
            }
        );

        let min = cells.min * 0.25 - shift;
        let max = cells.max * 0.25 - shift;
        let mut clamped = false;

        for point in grid3d(TyVector3F64::splat(-2.0), TyVector3F64::splat(2.0), 9) {
            let raw = shapes.evaluate(U32Id::from_u32(0), point).distance;
            let to_box = box_distance(point - (min + max) / 2.0, (max - min) / 2.0);
            let distance = lattice.evaluate(U32Id::from_u32(0), point).distance;

            assert_eq!(distance, raw.max(to_box));
            clamped |= to_box > raw;
        }

        assert!(clamped);
    }

    #[test]
    fn a_turned_box_rounds_out_to_the_lattice_box_around_its_corners() {
        let shapes = shapes_of(
            Vec::new(),
            vec![SdfShape3d::Box {
                min: TyVector3F64::ZERO,
                max: TyVector3F64::new(2.0, 1.0, 0.5),
                round: None,
            }],
        );
        let pose = TyPoseF64::new(
            TyVector3F64::new(0.0, 0.0, 1.0),
            TyQuaternionF64::from_rotation_y(FRAC_PI_2),
        );
        let lattice = LatticeShapes::new(&shapes, 0.5, pose);

        // The quarter turn about y carries +x onto -z and +z onto +x.
        assert_eq!(
            lattice.cells(U32Id::from_u32(0)).unwrap(),
            Bounds3d {
                min: TyVector3F64::new(0.0, 0.0, -2.0),
                max: TyVector3F64::new(1.0, 2.0, 2.0),
            }
        );
    }

    #[test]
    fn a_union_and_a_smooth_union_read_the_same_bits_as_their_folds() {
        // Copies of the origin sphere land on some of the spread spheres. A
        // copy reads its sphere's distance in another frame, so each tie shows
        // which child wins it.
        let sphere = |center: TyVector3F64, radius| SdfShape3d::Sphere { center, radius };
        let at = |index: u32| {
            TyVector3F64::new(
                f64::from(index % 5) * 1.25 - 2.5,
                f64::from(index % 3) * 0.75 - 0.75,
                f64::from(index / 5) * 1.25 - 2.5,
            )
        };

        let mut shapes = vec![sphere(TyVector3F64::ZERO, 0.5)];
        shapes.extend((0..25).map(|index| sphere(at(index), 0.3 + f64::from(index % 3) * 0.1)));
        shapes.extend((0..25).step_by(3).map(|index| SdfShape3d::Translate {
            shape_id: id(0),
            offset: at(index),
        }));

        let copies = (26..shapes.len() as u32).map(id);
        let children: Vec<U32Id<BSdfShape3d>> = copies
            .clone()
            .take(4)
            .chain((1..26).map(id))
            .chain(copies.skip(4))
            .collect();
        let union_id = id(shapes.len() as u32);
        let smooth_id = id(shapes.len() as u32 + 1);
        let radius = 0.4;

        shapes.push(SdfShape3d::Union {
            shape_ids: children.clone(),
        });
        shapes.push(SdfShape3d::SmoothUnion {
            radius,
            shape_ids: children.clone(),
        });

        let shapes = shapes_of(Vec::new(), shapes);
        let pose = TyPoseF64::new(
            TyVector3F64::new(0.1, 0.0, -0.1),
            TyQuaternionF64::from_rotation_y(0.3),
        );
        let lattice = LatticeShapes::new(&shapes, 0.25, pose);

        for point in grid3d(TyVector3F64::splat(-4.0), TyVector3F64::splat(4.0), 17) {
            let evaluations = children
                .iter()
                .map(|shape_id| lattice.evaluate(*shape_id, point));
            let nearer = |nearest: SdfEvaluation, evaluation: SdfEvaluation| {
                if evaluation.distance < nearest.distance {
                    evaluation
                } else {
                    nearest
                }
            };

            let union = evaluations.clone().reduce(nearer).unwrap();
            let mut rest = evaluations;
            let first = rest.next().unwrap();
            let (distance, nearest) = rest.fold(
                (first.distance, first),
                |(distance, nearest), evaluation| {
                    (
                        smooth_union_distance(distance, evaluation.distance, radius),
                        nearer(nearest, evaluation),
                    )
                },
            );
            let smooth = SdfEvaluation {
                distance,
                ..nearest
            };

            let clamp = |evaluation: SdfEvaluation, shape3d_id| {
                let cells = lattice.cells(shape3d_id).unwrap();
                let min = cells.min * 0.25 - pose.position;
                let max = cells.max * 0.25 - pose.position;
                let to_box =
                    box_distance(pose.rotation * point - (min + max) / 2.0, (max - min) / 2.0);

                SdfEvaluation {
                    distance: evaluation.distance.max(to_box),
                    ..evaluation
                }
            };

            assert!(
                same(lattice.evaluate(union_id, point), clamp(union, union_id)),
                "union at {point:?}"
            );
            assert!(
                same(lattice.evaluate(smooth_id, point), clamp(smooth, smooth_id)),
                "smooth union at {point:?}"
            );
        }

        assert!(lattice.tree_slots[union_id.to_usize_id()].is_some());
        assert!(lattice.trees.iter().all(|tree| tree.get().is_some()));
    }
}
