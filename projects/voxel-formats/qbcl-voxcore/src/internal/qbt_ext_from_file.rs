use crate::{QbtExt, QbtExtNode};
use qbcl::qbt::QbtFile;

/// The ext of `file`'s header around `nodes`, one entry per hierarchy node
/// in listing order.
pub fn qbt_ext_from_file(file: &QbtFile, nodes: Vec<Option<QbtExtNode>>) -> QbtExt {
    QbtExt {
        version: file.version,
        global_scale: file.global_scale,
        color_map: file
            .color_map
            .iter()
            .map(|color| [color.r, color.g, color.b, color.a])
            .collect(),
        nodes,
    }
}
