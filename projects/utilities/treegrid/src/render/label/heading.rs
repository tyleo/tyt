use std::num::NonZeroU8;

/// A markdown heading line at `level + depth`: a `#` run through
/// level `6`, markdown's deepest; past that, a bold label line
/// (`**text**`).
pub(crate) fn heading(level: NonZeroU8, depth: usize, text: &str) -> String {
    let level = usize::from(level.get()) + depth;
    if level <= 6 {
        format!("{} {text}", "#".repeat(level))
    } else {
        format!("**{text}**")
    }
}

#[cfg(test)]
mod tests {
    use crate::render;
    use std::num::NonZeroU8;

    #[test]
    fn depth_deepens_the_shallowest_level() {
        assert_eq!(
            render::heading(NonZeroU8::MIN, 1, "transform"),
            "## transform"
        );
    }

    #[test]
    fn level_six_is_the_deepest_hash_run() {
        assert_eq!(
            render::heading(NonZeroU8::new(6).unwrap(), 0, "transform"),
            "###### transform"
        );
    }

    #[test]
    fn a_heading_past_level_six_renders_bold() {
        assert_eq!(
            render::heading(NonZeroU8::new(6).unwrap(), 1, "transform"),
            "**transform**"
        );
    }
}
