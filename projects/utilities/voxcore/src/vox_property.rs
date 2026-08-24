use crate::BVoxValuePool;
use branded_id::U32Id;

/// One property in a [`VoxPalette`](crate::VoxPalette).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoxProperty {
    /// The property name: a neutral free string. A consumer ignores names it
    /// does not recognize.
    pub name: String,

    /// The value pool this property's materials draw values from.
    pub value_pool_id: U32Id<BVoxValuePool>,
}

#[cfg(test)]
mod tests {
    use crate::{BVoxValuePool, VoxProperty};
    use branded_id::U32Id;

    #[test]
    fn holds_a_name_and_a_value_pool() {
        let property = VoxProperty {
            name: "baseColor".to_owned(),
            value_pool_id: U32Id::<BVoxValuePool>::from_u32(2),
        };

        assert_eq!(property.name, "baseColor");
        assert_eq!(property.value_pool_id, U32Id::from_u32(2));
    }
}
