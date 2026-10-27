/// One occurrence of `--value` or `--values-from`, kept in line order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProgramFlag {
    Value(String),

    ValuesFrom(String),
}
