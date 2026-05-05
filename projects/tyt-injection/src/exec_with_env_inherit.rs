use std::{
    ffi::OsString,
    io::Result,
    process::{Command, Stdio},
};

/// Spawns `program` with the given args and additional env vars, inheriting
/// stdin/stdout/stderr from the parent. Waits for the child and returns its
/// exit code (or `-1` if the process was killed by a signal).
pub fn exec_with_env_inherit(
    program: &str,
    args: &[OsString],
    env: &[(OsString, OsString)],
) -> Result<i32> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    Ok(cmd.status()?.code().unwrap_or(-1))
}
