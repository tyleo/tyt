use crate::operations::object::CheckedDestination;
use vox_value_language::{CheckedProgram, EvaluatedProgram};

/// A checked record evaluated over one geometry.
pub struct ProgramRun<'a> {
    pub checked: &'a CheckedProgram,

    pub evaluated: EvaluatedProgram,

    pub destinations: &'a [CheckedDestination],
}
