use crate::render;

/// Pads `value` on the right with spaces to a visible width of `width`,
/// measuring past ANSI escapes.
pub(crate) fn pad_right(value: &str, width: usize) -> String {
    let visible = render::visible_width(value);
    let mut output = value.to_string();
    if width > visible {
        output.push_str(&" ".repeat(width - visible));
    }
    output
}

#[cfg(test)]
mod tests {
    use crate::render;

    #[test]
    fn pad_right_measures_past_escapes() {
        assert_eq!(render::pad_right("\x1b[0mab", 4), "\x1b[0mab  ");
    }

    #[test]
    fn pad_right_leaves_a_wide_value_unpadded() {
        assert_eq!(render::pad_right("abcdef", 3), "abcdef");
    }
}
