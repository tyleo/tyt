use crate::CliValue;
use voxsmith::operations::object::KeepRule;

impl CliValue for KeepRule {
    const VARIANTS: &'static [Self] = &[KeepRule::Any, KeepRule::Majority, KeepRule::All];

    fn name(self) -> &'static str {
        match self {
            KeepRule::All => "all",
            KeepRule::Any => "any",
            KeepRule::Majority => "majority",
        }
    }

    fn help(self) -> &'static str {
        match self {
            KeepRule::All => "Every cell of the block is live",
            KeepRule::Any => "Any cell of the block is live",
            KeepRule::Majority => "At least half of the block's cells are live",
        }
    }
}
