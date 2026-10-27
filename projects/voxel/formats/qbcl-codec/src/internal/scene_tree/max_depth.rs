/// The deepest the reader will descend, so a pathologically nested file is
/// rejected rather than overflowing the stack.
pub const MAX_DEPTH: usize = 4096;
