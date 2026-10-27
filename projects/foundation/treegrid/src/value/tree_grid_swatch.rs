use crate::TreeGridVisual;

/// The color block the default policy renders as a value's visual,
/// shown beside or instead of the value's text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TreeGridSwatch {
    /// A truecolor swatch of the given `[r, g, b]` bytes.
    Color([u8; 3]),

    /// A grayscale swatch: the level repeated on all three channels.
    Gray(u8),
}

impl TreeGridSwatch {
    /// Renders the canonical swatch visual: two cells of ANSI
    /// truecolor background.
    pub fn render(&self) -> TreeGridVisual {
        let [r, g, b] = match self {
            Self::Color(rgb) => *rgb,
            Self::Gray(level) => [*level; 3],
        };
        TreeGridVisual::new(format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m"), 2)
    }
}

#[cfg(test)]
mod tests {
    use crate::TreeGridSwatch;

    #[test]
    fn a_color_swatch_renders_two_background_cells() {
        let visual = TreeGridSwatch::Color([255, 0, 128]).render();

        assert_eq!(visual.rendered, "\x1b[48;2;255;0;128m  \x1b[0m");
        assert_eq!(visual.width, 2);
    }

    #[test]
    fn a_gray_swatch_repeats_the_level_on_all_channels() {
        let visual = TreeGridSwatch::Gray(128).render();

        assert_eq!(visual.rendered, "\x1b[48;2;128;128;128m  \x1b[0m");
        assert_eq!(visual.width, 2);
    }
}
