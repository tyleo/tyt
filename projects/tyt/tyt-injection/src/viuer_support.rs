use crossterm::{
    event::{poll, read},
    terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled},
};
use std::time::Duration;
use viuer::Config;

/// The base viuer config shared by every inline-image render: anchored at the
/// cursor rather than the terminal origin and, on Windows, skipping the
/// Kitty/iTerm2 capability probes that hang there.
pub(crate) fn base_config() -> Config {
    Config {
        absolute_offset: false,
        // No Windows terminal supports the Kitty graphics or iTerm2 inline-image
        // protocols, but viuer still probes for them by writing escape sequences
        // and blocking on `read_key` for the reply. Windows Terminal never
        // answers, so the probe hangs until the user presses a key. Skip the
        // probes on Windows; Sixel and the ANSI fallback are still tried.
        #[cfg(windows)]
        use_kitty: false,
        #[cfg(windows)]
        use_iterm: false,
        ..Default::default()
    }
}

/// Runs `print` — which writes terminal graphics escape sequences — with raw
/// mode asserted around it so capability-probe replies (Kitty graphics ACKs,
/// Primary Device Attributes, etc.) are not echoed back as visible garbage.
/// viuer toggles raw mode internally, so it is re-asserted afterwards and any
/// straggling reply bytes are drained before the original mode is restored.
pub(crate) fn with_capability_probe_guard<T>(print: impl FnOnce() -> T) -> T {
    let was_raw = is_raw_mode_enabled().unwrap_or(false);
    let _ = enable_raw_mode();

    let result = print();

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
