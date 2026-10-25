use crate::operations::sdf_doc::SdfShapes;
use branded_id::IdVec;
use sdfcore::{SdfMain, SdfShape2d, SdfShape3d, SdfState};
use std::collections::HashMap;

/// The shapes of a model holding `shapes2d` and `shapes3d`.
pub fn shapes_of(shapes2d: Vec<SdfShape2d>, shapes3d: Vec<SdfShape3d>) -> SdfShapes {
    let state = SdfState {
        shapes2d: IdVec::from_vec(shapes2d),
        shapes3d: IdVec::from_vec(shapes3d),
        ..SdfState::default()
    };

    SdfShapes::new(&SdfMain::new(state).unwrap(), HashMap::new())
}
