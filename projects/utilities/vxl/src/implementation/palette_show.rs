use crate::{Format, Result, Width, implementation};
use std::{num::NonZeroU8, path::Path};
use voxsmith::{
    PaletteShowLabel, PaletteShowLayout, PaletteShowOptions, PaletteShowTableShape,
    PropertySelector, render_palette_show,
};

/// Loads the voxel file at `input` and prints the value collections named by
/// `selectors`, each a property's values down a palette, rendered under
/// `layout`.
#[allow(clippy::too_many_arguments)]
pub fn palette_show(
    input: &Path,
    from: Option<Format>,
    selectors: &[PropertySelector],
    layout: PaletteShowLayout,
    label: Option<PaletteShowLabel>,
    header_level: Option<NonZeroU8>,
    table_shape: Option<PaletteShowTableShape>,
    width: Width,
) -> Result<()> {
    let state = implementation::load_state(input, from)?;

    let options = PaletteShowOptions {
        layout,
        label,
        header_level,
        table_shape,
        width: resolve_width(width),
    };

    let output = render_palette_show(&state, selectors, &options)?;

    implementation::write_stdout(output.as_bytes())
}

/// The column budget a `Width` resolves to, or `None` for no wrapping: a fixed
/// count, the terminal width, or unlimited. A `Terminal` width with no terminal
/// on stdout, as when the output is piped, also resolves to no wrapping.
fn resolve_width(width: Width) -> Option<usize> {
    match width {
        Width::Unlimited => None,
        Width::Columns(columns) => Some(columns),
        Width::Terminal => terminal_columns(),
    }
}

/// The terminal's column count, read from stdout, or `None` when stdout is not
/// a terminal.
#[cfg(unix)]
fn terminal_columns() -> Option<usize> {
    use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};
    use std::mem;
    // Safety: winsize is plain data; ioctl fills it for the stdout fd, and the
    // result is read only when the call reports success.
    unsafe {
        let mut size: winsize = mem::zeroed();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size) == 0 && size.ws_col > 0 {
            Some(size.ws_col as usize)
        } else {
            None
        }
    }
}

/// No terminal-width detection off unix; the `rows` layout does not wrap.
#[cfg(not(unix))]
fn terminal_columns() -> Option<usize> {
    None
}
