use std::io::{self, IsTerminal, Result};

/// Reads all of stdin to a string.
///
/// Returns an empty string when stdin is an interactive terminal so callers
/// don't block waiting for input that was never piped in.
pub fn read_stdin() -> Result<String> {
    let stdin = io::stdin();
    if stdin.is_terminal() {
        return Ok(String::new());
    }
    io::read_to_string(stdin)
}
