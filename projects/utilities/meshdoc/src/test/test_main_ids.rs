use crate::{BMeshHierarchyNode, BMeshImage, BMeshMaterial, BMeshObject, BMeshTexture};
use branded_id::U32Id;

/// The ids [`test_main`](crate::test_main) retains.
pub struct TestMainIds {
    pub image_id: U32Id<BMeshImage>,

    pub texture_id: U32Id<BMeshTexture>,

    pub material_id: U32Id<BMeshMaterial>,

    pub object_id: U32Id<BMeshObject>,

    pub root_id: U32Id<BMeshHierarchyNode>,
}
