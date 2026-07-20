use crate::BVoxValuePool;
use branded_id::U32Id;

/// One array property in a palette: it names a property and the
/// [`VoxValuePool`](crate::VoxValuePool) that property's materials draw from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoxArrayProperty {
    /// The property name: a neutral free string. A consumer ignores names it
    /// does not recognize.
    pub name: String,

    /// The value pool this property's materials draw values from.
    pub pool: U32Id<BVoxValuePool>,
}

#[cfg(test)]
mod tests {
    use crate::{BVoxValuePool, VoxArrayProperty};
    use branded_id::U32Id;

    #[test]
    fn holds_a_name_and_a_pool() {
        let property = VoxArrayProperty {
            name: "baseColorFactor".to_owned(),
            pool: U32Id::<BVoxValuePool>::from_u32(2),
        };

        assert_eq!(property.name, "baseColorFactor");
        assert_eq!(property.pool, U32Id::from_u32(2));
    }
}
