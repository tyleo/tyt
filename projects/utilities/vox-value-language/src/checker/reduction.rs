/// How a reduction combines the entries it gathers.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Reduction {
    Avg,
    Max,
    Min,
    Sum,
}
