/// A distinct, valid UUID for an extra object on a node placing several,
/// stamping the object's slot into the node id's fourth group. A node id keeps
/// that group zero, so this never collides with a node or another slot.
pub(crate) fn secondary_uuid(node_id: &str, slot: usize) -> String {
    match node_id.split('-').collect::<Vec<_>>().as_slice() {
        [a, b, c, _, e] => format!("{a}-{b}-{c}-{slot:04X}-{e}"),
        _ => node_id.to_owned(),
    }
}
