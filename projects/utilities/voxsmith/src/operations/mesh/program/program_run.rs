use crate::{
    Result,
    operations::mesh::{
        CheckedDestination, Destination, MeshEnvironment, MeshGeometry, MeshRecord, Swatches,
        run_program,
    },
};
use vox_value_language::{CheckedProgram, EvaluatedProgram};
use voxcore::VoxObject;

/// The program run over one geometry, with every destination checked in
/// its end scope.
pub(crate) struct ProgramRun {
    pub checked: CheckedProgram,
    pub evaluated: EvaluatedProgram,
    pub destinations: Vec<CheckedDestination>,
}

impl ProgramRun {
    /// Runs `record` over `geometry`.
    pub(crate) fn over(
        object: &VoxObject,
        swatches: &Swatches<'_>,
        record: &MeshRecord,
        geometry: &MeshGeometry,
    ) -> Result<Self> {
        let environment =
            MeshEnvironment::bind(object, swatches, geometry, &record.computed_bindings)?;

        let (checked, evaluated) = run_program(&record.program, &environment)?;

        let destinations = Destination::of_record(record)?
            .into_iter()
            .map(|destination| destination.check(&checked))
            .collect::<Result<_>>()?;

        Ok(ProgramRun {
            checked,
            evaluated,
            destinations,
        })
    }
}
