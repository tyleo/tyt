use crate::ext::{QbtExt, QbtExtNode};
use qbcl::qbt::QbtFile;
use voxcore::{VoxExt, VoxMain};

/// Puts the ext a read yields on the bare state. [`QbtExt`] keeps it. `()`
/// drops it.
pub trait QbtExtSink: VoxExt + Sized {
    /// Puts on `state` the ext of a read of `file`. `nodes` holds each
    /// hierarchy node's entry in listing order.
    fn record_file(state: VoxMain<()>, file: &QbtFile, nodes: Vec<QbtExtNode>) -> VoxMain<Self>;
}

impl QbtExtSink for QbtExt {
    fn record_file(state: VoxMain<()>, file: &QbtFile, nodes: Vec<QbtExtNode>) -> VoxMain<Self> {
        state.put_ext(QbtExt {
            version: file.version,
            global_scale: file.global_scale,
            color_map: file
                .color_map
                .iter()
                .map(|color| [color.r, color.g, color.b, color.a])
                .collect(),
            nodes: nodes.into_iter().map(Some).collect(),
        })
    }
}

impl QbtExtSink for () {
    fn record_file(state: VoxMain<()>, _file: &QbtFile, _nodes: Vec<QbtExtNode>) -> VoxMain<Self> {
        state
    }
}
