use crate::{
    CheckedProgram, EvaluatedProgram, Result, ValueEnvironment, check, eval, parse, types_of,
};

/// Checks and evaluates the program over the environment.
pub fn run(
    text: &str,
    environment: &ValueEnvironment,
) -> Result<(CheckedProgram, EvaluatedProgram)> {
    let checked = check(parse(text)?, &types_of(environment))?;
    let evaluated = eval(&checked, environment)?;

    Ok((checked, evaluated))
}
