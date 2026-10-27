use crate::CliValue;

/// Where the rendered views go.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputKind {
    /// Inline in the terminal.
    Terminal,

    /// A PNG per view beside the input.
    Png,
}

impl CliValue for OutputKind {
    const VARIANTS: &'static [Self] = &[OutputKind::Terminal, OutputKind::Png];

    fn name(self) -> &'static str {
        match self {
            OutputKind::Terminal => "terminal",
            OutputKind::Png => "png",
        }
    }

    fn help(self) -> &'static str {
        match self {
            OutputKind::Terminal => "Inline in the terminal",
            OutputKind::Png => "A PNG per view beside the input",
        }
    }
}
