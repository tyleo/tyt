use crate::{
    VmaxFileBuilder,
    ext::{VMaxExt, VMaxVoxMain},
};

impl<'a> VmaxFileBuilder<'a, Option<VMaxExt>> {
    /// Starts a builder writing `state` back through its ext, the typed form
    /// of [`new`](VmaxFileBuilder::new). A state carrying no ext writes a
    /// synthesized document.
    pub fn new_with_ext(state: &'a VMaxVoxMain) -> Self {
        Self::with_ext(state, state.ext().as_ref())
    }
}
