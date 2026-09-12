use mvox::MVoxFrame;
use ty_math::{
    TyMatrix4x4F64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32,
};

/// The transform projecting a transform node's first frame, or the identity
/// when it has none. The exact frames ride in the ext. The loader sets a node's
/// transform from this and the writer checks that it still holds.
pub fn transform_from_frames(frames: &[MVoxFrame]) -> TyTransformF64 {
    match frames.first() {
        Some(frame) => transform_from_frame(frame),
        None => TyTransformF64::default(),
    }
}

/// Projects one keyframe to a [`TyTransformF64`]. The rotation is the frame's
/// signed-permutation matrix. An improper one, a mirror, splits into a proper
/// rotation and a negative x scale, which keeps voxcore's unit-quaternion
/// invariant.
fn transform_from_frame(frame: &MVoxFrame) -> TyTransformF64 {
    let position = TyVector3I32::from_array(frame.translation).as_dvec3();

    let signed = frame.rotation.to_matrix();
    let mut matrix = [[0.0f64; 3]; 3];
    for row in 0..3 {
        for column in 0..3 {
            matrix[row][column] = signed[row][column] as f64;
        }
    }

    let mut scale = TyVector3F64::new(1.0, 1.0, 1.0);
    if determinant(&matrix) < 0.0 {
        // M = R * diag(-1, 1, 1), so negating column 0 leaves a proper
        // rotation.
        for row in &mut matrix {
            row[0] = -row[0];
        }
        scale.x = -1.0;
    }

    // Read the proper rotation into a column-major matrix and decode it. The
    // frame is a signed permutation, so after the mirror split it is always a
    // proper rotation.
    let rotation = TyMatrix4x4F64::from_cols_array_2d(&[
        [matrix[0][0], matrix[1][0], matrix[2][0], 0.0],
        [matrix[0][1], matrix[1][1], matrix[2][1], 0.0],
        [matrix[0][2], matrix[1][2], matrix[2][2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);
    let rotation = TyQuaternionF64::from_rotation_matrix(rotation)
        .expect("the frame is a proper rotation after the mirror split");

    TyTransformF64::new(position, rotation, scale)
}

/// The determinant of a 3x3 matrix, the scalar triple product of its columns.
fn determinant(matrix: &[[f64; 3]; 3]) -> f64 {
    let column = |c: usize| TyVector3F64::new(matrix[0][c], matrix[1][c], matrix[2][c]);
    column(0).dot(column(1).cross(column(2)))
}
