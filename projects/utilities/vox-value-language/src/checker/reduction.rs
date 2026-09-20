use crate::Domain;

/// How a reduction combines the entries it gathers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Reduction {
    Avg,
    Max,
    Min,
    Sum,
}

impl Reduction {
    /// The function that reduces into the domain, `max` for plain and
    /// `swatchMax` for a swatch.
    pub(crate) fn name(self, target: Domain) -> String {
        let (plain, suffix) = match self {
            Reduction::Avg => ("avg", "Avg"),
            Reduction::Max => ("max", "Max"),
            Reduction::Min => ("min", "Min"),
            Reduction::Sum => ("sum", "Sum"),
        };

        match target {
            Domain::Plain => plain.to_owned(),
            target => format!("{target}{suffix}"),
        }
    }
}
