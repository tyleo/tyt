/// The properties materials have to agree on to merge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PartitionProperties {
    /// Every property other than the quantized one.
    All,

    /// The named properties, none when empty.
    Named(Vec<String>),
}
