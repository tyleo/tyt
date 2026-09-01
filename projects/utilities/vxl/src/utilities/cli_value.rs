/// A voxsmith enum the command line takes by name. Each variant has the name a
/// user types and the help line `--help` prints beside it;
/// [`cli_value_parser`](crate::cli_value_parser) turns the set into a clap
/// value parser.
pub trait CliValue: Copy + Send + Sync + 'static {
    /// Every variant, in the order `--help` lists them.
    const VARIANTS: &'static [Self];

    /// The name a user types for this variant.
    fn name(self) -> &'static str;

    /// The help line `--help` prints beside the name.
    fn help(self) -> &'static str;

    /// Parses a name, listing the accepted names on failure.
    fn parse(text: &str) -> Result<Self, String> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|variant| variant.name() == text)
            .ok_or_else(|| {
                let names: Vec<_> = Self::VARIANTS
                    .iter()
                    .map(|variant| variant.name())
                    .collect();

                format!("`{text}` is not one of {}", names.join(", "))
            })
    }
}
