/// A syntactically valid, deterministic UUID for a synthesized scene node.
/// Voxel Max decodes a node's `id` and `pid` as a UUID and rejects a non-UUID
/// token. The index is offset by one so the first node avoids the all-zero nil
/// UUID.
pub fn synth_uuid(index: usize) -> String {
    format!("00000000-0000-0000-0000-{:012X}", index + 1)
}
