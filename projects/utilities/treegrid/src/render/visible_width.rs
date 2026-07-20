/// The visible width of `value`, counting characters outside of ANSI CSI escape
/// sequences so a visual's color codes carry no width.
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

#[cfg(test)]
mod tests {
    use crate::render;

    #[test]
    fn visible_width_counts_plain_characters() {
        assert_eq!(render::visible_width("#FF0000FF"), 9);
    }

    #[test]
    fn visible_width_skips_csi_sequences() {
        assert_eq!(render::visible_width("\x1b[48;2;255;0;128m  \x1b[0m"), 2);
    }

    #[test]
    fn visible_width_counts_text_around_escapes() {
        assert_eq!(render::visible_width("a\x1b[48;2;0;0;0m  \x1b[0mb"), 4);
    }
}
