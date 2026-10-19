use branded_id::U32Id;
use sdfcore::{BSdfShape2d, BSdfShape3d, SdfShape3d};

/// The 3D shapes and the profile that `shape` references, in argument order.
pub fn shape3d_references(
    shape: &SdfShape3d,
) -> (Vec<U32Id<BSdfShape3d>>, Option<U32Id<BSdfShape2d>>) {
    match shape {
        SdfShape3d::Bend { shape_id, .. }
        | SdfShape3d::Displace { shape_id, .. }
        | SdfShape3d::Elongate { shape_id, .. }
        | SdfShape3d::Mirror { shape_id, .. }
        | SdfShape3d::Offset { shape_id, .. }
        | SdfShape3d::Orient { shape_id, .. }
        | SdfShape3d::Repeat { shape_id, .. }
        | SdfShape3d::RepeatPolar { shape_id, .. }
        | SdfShape3d::Rotate { shape_id, .. }
        | SdfShape3d::Scale { shape_id, .. }
        | SdfShape3d::Shell { shape_id, .. }
        | SdfShape3d::Translate { shape_id, .. }
        | SdfShape3d::Twist { shape_id, .. } => (vec![*shape_id], None),

        SdfShape3d::Box { .. }
        | SdfShape3d::BoxFrame { .. }
        | SdfShape3d::Capsule { .. }
        | SdfShape3d::Cone { .. }
        | SdfShape3d::Cylinder { .. }
        | SdfShape3d::Ellipsoid { .. }
        | SdfShape3d::HalfSpace { .. }
        | SdfShape3d::Lathe { .. }
        | SdfShape3d::Octahedron { .. }
        | SdfShape3d::Pyramid { .. }
        | SdfShape3d::RoundCone { .. }
        | SdfShape3d::Sphere { .. }
        | SdfShape3d::Torus { .. } => (Vec::new(), None),

        SdfShape3d::Extrude { profile_id, .. }
        | SdfShape3d::Revolve { profile_id, .. }
        | SdfShape3d::Sweep { profile_id, .. } => (Vec::new(), Some(*profile_id)),

        SdfShape3d::Intersect { shape_ids }
        | SdfShape3d::SmoothIntersect { shape_ids, .. }
        | SdfShape3d::SmoothUnion { shape_ids, .. }
        | SdfShape3d::Union { shape_ids } => (shape_ids.clone(), None),

        SdfShape3d::SmoothSubtract {
            base_id,
            cutter_ids,
            ..
        }
        | SdfShape3d::Subtract {
            base_id,
            cutter_ids,
        } => (
            [*base_id]
                .into_iter()
                .chain(cutter_ids.iter().copied())
                .collect(),
            None,
        ),
    }
}
