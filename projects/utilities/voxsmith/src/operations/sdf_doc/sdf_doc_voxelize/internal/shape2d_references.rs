use branded_id::U32Id;
use sdfcore::{BSdfShape2d, SdfShape2d};

/// The 2D shapes that `shape` references, in argument order.
pub fn shape2d_references(shape: &SdfShape2d) -> Vec<U32Id<BSdfShape2d>> {
    match shape {
        SdfShape2d::Arc { .. }
        | SdfShape2d::Arch { .. }
        | SdfShape2d::Circle { .. }
        | SdfShape2d::Ellipse { .. }
        | SdfShape2d::Ngon { .. }
        | SdfShape2d::Polygon { .. }
        | SdfShape2d::Polyline { .. }
        | SdfShape2d::Rect { .. }
        | SdfShape2d::Sector { .. }
        | SdfShape2d::Spline { .. }
        | SdfShape2d::Star { .. }
        | SdfShape2d::Vesica { .. } => Vec::new(),

        SdfShape2d::Intersect { shape_ids }
        | SdfShape2d::SmoothIntersect { shape_ids, .. }
        | SdfShape2d::SmoothUnion { shape_ids, .. }
        | SdfShape2d::Union { shape_ids } => shape_ids.clone(),

        SdfShape2d::Mirror { shape_id, .. }
        | SdfShape2d::Offset { shape_id, .. }
        | SdfShape2d::Repeat { shape_id, .. }
        | SdfShape2d::RepeatPolar { shape_id, .. }
        | SdfShape2d::Rotate { shape_id, .. }
        | SdfShape2d::Scale { shape_id, .. }
        | SdfShape2d::Shell { shape_id, .. }
        | SdfShape2d::Translate { shape_id, .. } => vec![*shape_id],

        SdfShape2d::SmoothSubtract {
            base_id,
            cutter_ids,
            ..
        }
        | SdfShape2d::Subtract {
            base_id,
            cutter_ids,
        } => [*base_id]
            .into_iter()
            .chain(cutter_ids.iter().copied())
            .collect(),
    }
}
