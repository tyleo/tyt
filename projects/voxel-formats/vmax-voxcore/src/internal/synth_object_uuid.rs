/// A syntactically valid, deterministic UUID for a synthesized object's
/// contents file, in a lane of its own so it never collides with a node's
/// [`synth_uuid`](crate::synth_uuid).
pub fn synth_object_uuid(index: usize) -> String {
    format!("00000000-0000-0001-0000-{:012X}", index + 1)
}
