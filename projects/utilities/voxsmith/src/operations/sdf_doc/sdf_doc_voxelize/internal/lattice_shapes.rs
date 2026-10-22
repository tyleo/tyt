use crate::operations::sdf_doc::{Bounds3d, SdfEvaluation, SdfShapes, ShapeSource, box_distance};
use branded_id::{IdVec, U32Id};
use sdfcore::{BSdfShape2d, BSdfShape3d};
use ty_math::{TyPoseF64, TyQuaternionF64, TyVector2F64, TyVector3F64};

/// How far a turned box's corner can sit from a cell corner, in cells, and
/// still count as on it. A quarter turn's rounding then adds no empty layer.
const TURNED_CORNER_TOLERANCE: f64 = 1e-9;

/// A model's shapes posed onto the lattice of cells `voxel_size` across. Each
/// 3D shape's distance takes the larger of its formula's value and the signed
/// distance to the lattice box around its posed box, rounded out to the
/// lattice's cell corners.
#[derive(Clone, Debug)]
pub struct LatticeShapes<'a> {
    shapes: &'a SdfShapes,

    voxel_size: f64,

    pose: TyPoseF64,

    /// Each box's corners as whole numbers of cells in the lattice.
    cells: IdVec<BSdfShape3d, Option<Bounds3d>>,
}

impl<'a> LatticeShapes<'a> {
    /// `shapes` posed by `pose` onto the lattice of cells `voxel_size` across.
    /// `pose` takes a shape's point to its lattice point in meters.
    pub fn new(shapes: &'a SdfShapes, voxel_size: f64, pose: TyPoseF64) -> Self {
        let cells = shapes
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

        Self {
            shapes,
            voxel_size,
            pose,
            cells,
        }
    }

    /// The rounded box of the shape at `shape3d_id`, with its corners as whole
    /// numbers of cells in the lattice, or `None` for a shape that reaches
    /// without end.
    pub fn cells(&self, shape3d_id: U32Id<BSdfShape3d>) -> Option<Bounds3d> {
        self.cells[shape3d_id.to_usize_id()]
    }
}

impl ShapeSource for LatticeShapes<'_> {
    fn evaluate(&self, shape3d_id: U32Id<BSdfShape3d>, point: TyVector3F64) -> SdfEvaluation {
        let evaluation = self.shapes.field3d(shape3d_id).evaluate(self, point);

        match self.cells(shape3d_id) {
            Some(cells) => {
                // The box on the lattice's axes, moved back by the pose's
                // position alone.
                let min = cells.min * self.voxel_size - self.pose.position;
                let max = cells.max * self.voxel_size - self.pose.position;
                let to_box = box_distance(
                    self.pose.rotation * point - (min + max) / 2.0,
                    (max - min) / 2.0,
                );

                SdfEvaluation {
                    distance: evaluation.distance.max(to_box),
                    ..evaluation
                }
            }

            None => evaluation,
        }
    }

    fn distance2d(&self, shape2d_id: U32Id<BSdfShape2d>, point: TyVector2F64) -> f64 {
        self.shapes.field2d(shape2d_id).distance(self, point)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        Bounds3d, LatticeShapes, ShapeSource, box_distance, grid3d, shapes_of,
    };
    use branded_id::U32Id;
    use sdfcore::SdfShape3d;
    use std::f64::consts::FRAC_PI_2;
    use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3F64};

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
}
