/// The visible width of `value`, counting characters outside of ANSI CSI escape
/// sequences so a swatch's color codes carry no width.
pub(crate) fn visible_width(value: &str) -> usize {
    let mut width = 0;
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character == '\x1b' {
            // A CSI sequence is `ESC [` then bytes up to a final `0x40..=0x7e`.
            if chars.next() == Some('[') {
                for tail in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&tail) {
                        break;
                    }
                }
            }
        } else {
            width += 1;
        }
    }
    width
}

/// Pads `value` on the right with spaces to a visible width of `width`,
/// measuring past the swatch escape codes.
pub(crate) fn pad_right(value: &str, width: usize) -> String {
    let visible = visible_width(value);
    let mut output = value.to_string();
    if width > visible {
        output.push_str(&" ".repeat(width - visible));
    }
    output
}
