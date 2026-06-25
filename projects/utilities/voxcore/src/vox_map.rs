use crate::VoxValue;

/// An ordered set of key/value pairs: the object form of a [`VoxValue`].
///
/// Insertion order is preserved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxMap(pub Vec<(String, VoxValue)>);
