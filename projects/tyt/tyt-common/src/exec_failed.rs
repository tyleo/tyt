/// Fields from a failed external command execution.
#[derive(Debug)]
pub struct ExecFailed {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
