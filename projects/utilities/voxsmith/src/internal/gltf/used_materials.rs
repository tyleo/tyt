use branded_id::U32Id;
use std::collections::HashMap;
use voxcore::{BVoxLayer, BVoxMaterial, BVoxPalette, BVoxVoxel, VoxObject};

/// The distinct materials an object's voxels sample in one layer, the set the
/// material atlas bakes.
///
/// A layer references a palette that holds many materials, and each live voxel
/// samples one of them, so across the whole object a single layer uses a set of
/// materials, not just one. This collects that set, the distinct materials the
/// object's voxels sample in the read layer, first seen in raster order, and
/// records which one each voxel sampled. The bake gives each distinct material
/// one atlas texel ([`len`](Self::len) of them), and each voxel's mesh faces
/// sample the texel of the material it uses (via
/// [`material_index`](Self::material_index)).
pub(crate) struct UsedMaterials {
    /// The palette the read layer references.
    palette: U32Id<BVoxPalette>,

    /// Distinct materials in first-seen order, one per atlas texel.
    materials: Vec<U32Id<BVoxMaterial>>,

    /// Per live voxel, the position in [`materials`](Self::materials), and so the
    /// atlas texel, of the material it samples.
    index_of: HashMap<U32Id<BVoxVoxel>, u32>,
}

impl UsedMaterials {
    /// The palette the read layer references.
    pub(crate) fn palette(&self) -> U32Id<BVoxPalette> {
        self.palette
    }

    /// The material at position `index` in the used set, or `None` when `index`
    /// is out of range.
    pub(crate) fn material(&self, index: usize) -> Option<U32Id<BVoxMaterial>> {
        self.materials.get(index).copied()
    }

    /// Number of distinct materials.
    pub(crate) fn len(&self) -> usize {
        self.materials.len()
    }

    /// The atlas texel `voxel` samples, its material's position in the used set,
    /// or `None` if `voxel` is not a live voxel of the resolved object.
    pub(crate) fn material_index(&self, voxel: U32Id<BVoxVoxel>) -> Option<u32> {
        self.index_of.get(&voxel).copied()
    }
}

/// Resolves the distinct materials `object` uses in layer `layer`, first seen in
/// raster order, or `None` when `layer` is not one of `object`'s layers and so
/// names no palette. Two voxels share a material exactly when they sample the
/// same material in that layer.
pub(crate) fn resolve_used_materials(
    object: &VoxObject,
    layer: U32Id<BVoxLayer>,
) -> Option<UsedMaterials> {
    let palette = object
        .iter_layers()
        .find(|&(id, _)| id == layer)
        .map(|(_, palette)| palette)?;

    let mut materials: Vec<U32Id<BVoxMaterial>> = Vec::new();
    let mut lookup: HashMap<U32Id<BVoxMaterial>, u32> = HashMap::new();
    let mut index_of: HashMap<U32Id<BVoxVoxel>, u32> = HashMap::new();

    for voxel in object.iter_live() {
        let material = object
            .voxel_material(voxel, layer)
            .expect("a live voxel samples a material in one of the object's layers");

        let index = *lookup.entry(material).or_insert_with(|| {
            let index = materials.len() as u32;
            materials.push(material);
            index
        });

        index_of.insert(voxel, index);
    }

    Some(UsedMaterials {
        palette,
        materials,
        index_of,
    })
}
