use ty_math::TyVector3;

/// Parsed mesh data: vertices, triangle indices, and per-triangle UV coordinates.
pub type MeshWithUvs = (Vec<TyVector3>, Vec<[usize; 3]>, Vec<[[f64; 2]; 3]>);
