use crate::CliValue;
use voxsmith::operations::object::QuarterTurns;

impl CliValue for QuarterTurns {
    const VARIANTS: &'static [Self] = &[QuarterTurns::One, QuarterTurns::Two, QuarterTurns::Three];

    fn name(self) -> &'static str {
        match self {
            QuarterTurns::One => "1",
            QuarterTurns::Two => "2",
            QuarterTurns::Three => "3",
        }
    }

    fn help(self) -> &'static str {
        match self {
            QuarterTurns::One => "A quarter turn",
            QuarterTurns::Two => "A half turn",
            QuarterTurns::Three => "Three quarter turns",
        }
    }
}
