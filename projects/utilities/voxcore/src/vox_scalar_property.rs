use crate::{BVoxPoolValue, BVoxValuePool};
use branded_id::U32Id;

/// One scalar property in a palette: it names a property and pins it to a
/// single value in a [`VoxValuePool`](crate::VoxValuePool), one value for the
/// whole palette with no per-material column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoxScalarProperty {
    /// The property name: a neutral free string. A consumer ignores names it
    /// does not recognize.
    pub name: String,

    /// The value pool the pinned value lives in.
    pub pool: U32Id<BVoxValuePool>,

    /// The value id of the pinned value in `pool`.
    pub value_id: U32Id<BVoxPoolValue>,
}

#[cfg(test)]
mod tests {
    use crate::{BVoxPoolValue, BVoxValuePool, VoxScalarProperty};
    use branded_id::U32Id;

    #[test]
    fn holds_a_name_a_pool_and_a_value_id() {
        let property = VoxScalarProperty {
            name: "emissiveStrength".to_owned(),
            pool: U32Id::<BVoxValuePool>::from_u32(2),
            value_id: U32Id::<BVoxPoolValue>::from_u32(5),
        };

        assert_eq!(property.name, "emissiveStrength");
        assert_eq!(property.pool, U32Id::from_u32(2));
        assert_eq!(property.value_id, U32Id::from_u32(5));
    }
}
