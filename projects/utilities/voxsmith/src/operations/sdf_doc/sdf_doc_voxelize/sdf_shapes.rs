use crate::operations::sdf_doc::{
    Bounds2d, Bounds3d, OutlineSegment, SdfEvaluation, Shape2dField, Shape3dField, ShapeSource,
};
use branded_id::{IdVec, IteratorExt, U32Id};
use sdfcore::{BSdfShape2d, BSdfShape3d, SdfMain, SdfShape2d};
use std::collections::HashMap;
use ty_math::{TyVector2F64, TyVector3F64};

/// A model's shapes, ready to evaluate at any point by the sdfj format's
/// distances.
#[derive(Clone, Debug)]
pub struct SdfShapes {
    shapes3d: IdVec<BSdfShape3d, Shape3dField>,

    bounds3d: IdVec<BSdfShape3d, Option<Bounds3d>>,

    shapes2d: IdVec<BSdfShape2d, Shape2dField>,

    bounds2d: IdVec<BSdfShape2d, Bounds2d>,
}

impl SdfShapes {
    /// The shapes of `main`, with every sine and cosine model evaluation takes
    /// once already computed. `text_outlines` holds the contours of each
    /// `text`.
    ///
    /// # Panics
    ///
    /// When a shape's arguments fail the sdfj format's validation.
    pub(crate) fn new(
        main: &SdfMain,
        mut text_outlines: HashMap<U32Id<BSdfShape2d>, Vec<Vec<OutlineSegment>>>,
    ) -> Self {
        let state = main.state();

        let mut shapes = Self {
            shapes3d: IdVec::with_capacity(state.shapes3d.len()),
            bounds3d: IdVec::with_capacity(state.shapes3d.len()),
            shapes2d: IdVec::with_capacity(state.shapes2d.len()),
            bounds2d: IdVec::with_capacity(state.shapes2d.len()),
        };

        for (shape2d_id, shape) in state.shapes2d.iter().enumerate_ids() {
            let field = match shape {
                SdfShape2d::Text { .. } => Shape2dField::Outline {
                    contours: text_outlines
                        .remove(&shape2d_id)
                        .expect("each text comes laid out"),
                },

                _ => Shape2dField::new(shape),
            };
            shapes.bounds2d.push(field.bounds(&shapes.bounds2d));
            shapes.shapes2d.push(field);
        }

        for shape in state.shapes3d.iter() {
            let field = Shape3dField::new(shape, &shapes.bounds3d);
            shapes
                .bounds3d
                .push(field.bounds(&shapes.bounds3d, &shapes.bounds2d));
            shapes.shapes3d.push(field);
        }

        shapes
    }

    /// The shape at `shape3d_id` evaluated at `point`.
    pub fn evaluate(&self, shape3d_id: U32Id<BSdfShape3d>, point: TyVector3F64) -> SdfEvaluation {
        self.shapes3d[shape3d_id.to_usize_id()].evaluate(self, point)
    }

    /// The box around the 3D shape at `shape3d_id`, before any rounding, or
    /// `None` for a shape that reaches without end.
    pub(crate) fn bounds3d(&self, shape3d_id: U32Id<BSdfShape3d>) -> Option<Bounds3d> {
        self.bounds3d[shape3d_id.to_usize_id()]
    }

    /// Each 3D shape's box, before any rounding.
    pub(crate) fn all_bounds3d(&self) -> &IdVec<BSdfShape3d, Option<Bounds3d>> {
        &self.bounds3d
    }

    /// The signed distance of the 2D shape at `shape2d_id` at `point`.
    pub(crate) fn distance2d(&self, shape2d_id: U32Id<BSdfShape2d>, point: TyVector2F64) -> f64 {
        self.shapes2d[shape2d_id.to_usize_id()].distance(self, point)
    }

    /// The field of the 3D shape at `shape3d_id`.
    pub(crate) fn field3d(&self, shape3d_id: U32Id<BSdfShape3d>) -> &Shape3dField {
        &self.shapes3d[shape3d_id.to_usize_id()]
    }

    /// The field of the 2D shape at `shape2d_id`.
    pub(crate) fn field2d(&self, shape2d_id: U32Id<BSdfShape2d>) -> &Shape2dField {
        &self.shapes2d[shape2d_id.to_usize_id()]
    }
}

impl ShapeSource for SdfShapes {
    fn evaluate(&self, shape3d_id: U32Id<BSdfShape3d>, point: TyVector3F64) -> SdfEvaluation {
        SdfShapes::evaluate(self, shape3d_id, point)
    }

    fn distance2d(&self, shape2d_id: U32Id<BSdfShape2d>, point: TyVector2F64) -> f64 {
        SdfShapes::distance2d(self, shape2d_id, point)
    }
}
