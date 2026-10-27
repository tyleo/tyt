use crate::{
    Error, Result,
    operations::object::{
        CheckedDestination, Destination, MeshElement, MeshEnvironment, MeshGeometry, MeshRecord,
        ProgramRun, Swatches,
    },
};
use vox_value_language::{CheckedProgram, check, eval, parse};
use voxcore::VoxObject;

/// A record's program and destinations, checked once against an object's
/// palette so that each geometry only evaluates them.
pub struct CheckedRecord {
    pub checked: CheckedProgram,

    pub destinations: Vec<CheckedDestination>,

    environment: MeshEnvironment,
}

impl CheckedRecord {
    /// Binds only the properties the program or a destination reads. A program
    /// error rises from the program element.
    pub(crate) fn check(swatches: &Swatches<'_>, record: &MeshRecord) -> Result<Self> {
        let parsed = parse(&record.program)
            .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

        let destinations = Destination::of_record(record)?;

        let free_names = parsed.free_names(
            destinations
                .iter()
                .map(|destination| &destination.expression),
        );

        let environment = MeshEnvironment::bind(swatches, &record.computed_bindings, &free_names)?;

        let checked = check(parsed, &environment.types)
            .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

        let destinations = destinations
            .into_iter()
            .map(|destination| destination.check(&checked))
            .collect::<Result<_>>()?;

        Ok(CheckedRecord {
            checked,
            destinations,
            environment,
        })
    }

    /// Evaluates the program over `object`'s `geometry`. An error rises from
    /// the program element.
    pub(crate) fn run(
        &self,
        object: &VoxObject,
        swatches: &Swatches<'_>,
        geometry: &MeshGeometry,
    ) -> Result<ProgramRun<'_>> {
        let values = self.environment.values(object, swatches, geometry);

        let evaluated = eval(&self.checked, &values)
            .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

        Ok(ProgramRun {
            checked: &self.checked,
            evaluated,
            destinations: &self.destinations,
        })
    }
}
