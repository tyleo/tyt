use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use std::{
    io::{Error as IOError, Result},
    path::Path,
    time::Duration,
};
use viuer::{Config, print_from_file};

/// Prints an image to the terminal using the Kitty, iTerm2, or Sixel graphics
/// protocol when supported, falling back to ANSI half-blocks otherwise.
pub fn display_image_in_terminal(path: &Path) -> Result<()> {
    let cfg = Config {
        absolute_offset: false,
        ..Default::default()
    };

    // Enable raw mode around viuer so terminal capability-probe replies
    // (Kitty graphics ACKs, Primary Device Attributes, etc.) are not echoed
    // back as visible garbage. viuer toggles raw mode internally, so we
    // re-assert it afterwards and drain any straggling reply bytes before
    // restoring the original mode.
    let was_raw = is_raw_mode_enabled().unwrap_or(false);
    let _ = enable_raw_mode();

    let result = print_from_file(path, &cfg)
        .map(|_| ())
        .map_err(IOError::other);

    let _ = enable_raw_mode();
    while matches!(poll(Duration::from_millis(50)), Ok(true)) {
        if read().is_err() {
            break;
        }
    }
    if !was_raw {
        let _ = disable_raw_mode();
    }

    result
}
