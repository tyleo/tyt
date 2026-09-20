use crate::{
    Error, Result,
    operations::mesh::{MeshElement, MeshEnvironment},
};
use vox_value_language::{CheckedProgram, EvaluatedProgram, check, eval, parse};

/// Parses, checks, and evaluates `program` over `environment`, every error
/// rising from the program element.
pub(crate) fn run_program(
    program: &str,
    environment: &MeshEnvironment,
) -> Result<(CheckedProgram, EvaluatedProgram)> {
    let parsed = parse(program).map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

    let checked = check(parsed, &environment.types)
        .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

    let evaluated = eval(&checked, &environment.values)
        .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

    Ok((checked, evaluated))
}
