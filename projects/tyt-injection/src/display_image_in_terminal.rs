use std::{
    io::{Error as IOError, Result},
    path::Path,
};
use viuer::{Config, print_from_file};

/// Prints an image to the terminal using the Kitty, iTerm2, or Sixel graphics
/// protocol when supported, falling back to ANSI half-blocks otherwise.
pub fn display_image_in_terminal(path: &Path) -> Result<()> {
    let cfg = Config {
        absolute_offset: false,
        ..Default::default()
    };
    print_from_file(path, &cfg)
        .map(|_| ())
        .map_err(IOError::other)
}
