use crate::CliValue;
use voxsmith::PaletteShowLabel;

impl CliValue for PaletteShowLabel {
    const VARIANTS: &'static [Self] = &[
        PaletteShowLabel::None,
        PaletteShowLabel::Concat,
        PaletteShowLabel::Header,
    ];

    fn name(self) -> &'static str {
        match self {
            PaletteShowLabel::None => "none",
            PaletteShowLabel::Concat => "concat",
            PaletteShowLabel::Header => "header",
        }
    }

    fn help(self) -> &'static str {
        match self {
            PaletteShowLabel::None => "No labels",
            PaletteShowLabel::Concat => "Full dot-joined paths, like `0.\"baseColor\".a`",
            PaletteShowLabel::Header => {
                "Nested markdown headings over value collections labeled by their leaf segment \
                 alone"
            }
        }
    }
}
