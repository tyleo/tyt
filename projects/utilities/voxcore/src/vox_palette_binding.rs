use crate::BVoxValuePool;
use branded_id::U32Id;

/// One attribute-to-pool binding in a palette: it names an attribute and the
/// [`VoxValuePool`](crate::VoxValuePool) that attribute's materials draw from.
///
/// Binding order fixes the column order of a palette's column-major materials,
/// and no two bindings in a palette share an attribute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoxPaletteBinding {
    /// The bound attribute name: a neutral free string. A consumer ignores
    /// names it does not recognize.
    pub attribute: String,

    /// The value pool this binding's attribute draws its values from.
    pub pool: U32Id<BVoxValuePool>,
}

#[cfg(test)]
mod tests {
    use crate::{BVoxValuePool, VoxPaletteBinding};
    use branded_id::U32Id;

    #[test]
    fn holds_an_attribute_and_a_pool_ref() {
        let binding = VoxPaletteBinding {
            attribute: "baseColorFactor".to_owned(),
            pool: U32Id::<BVoxValuePool>::from_u32(2),
        };

        assert_eq!(binding.attribute, "baseColorFactor");
        assert_eq!(binding.pool, U32Id::from_u32(2));
    }
}
