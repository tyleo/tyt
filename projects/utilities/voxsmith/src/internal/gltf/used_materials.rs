use branded_id::U32Id;
use std::collections::HashSet;
use voxcore::{BVoxPalette, BVoxPaletteCell, BVoxPaletteRef, VoxObject};

/// The distinct merged materials a meshed object actually uses.
///
/// A voxel's material is the tuple of cells it samples across the object's
/// palette references. Under the palette atlas each distinct tuple takes one
/// texel, so this collects the tuples the object uses, first seen in raster
/// order, giving the compact per-mesh material set. An object with no references
/// resolves to a single, all-default material every voxel shares.
pub(crate) struct UsedMaterials {
    /// The object's palette references in resolution order, paired with the
    /// palette each names, for reading a material's attribute values.
    references: Vec<(U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>)>,

    /// Distinct materials in first-seen order; each holds the cell sampled from
    /// every reference, aligned with [`references`](Self::references).
    materials: Vec<Vec<U32Id<BVoxPaletteCell>>>,
}

impl UsedMaterials {
    /// The object's palette references in resolution order.
    pub(crate) fn references(&self) -> &[(U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>)] {
        &self.references
    }

    /// The cells the material at `index` samples, aligned with
    /// [`references`](Self::references).
    pub(crate) fn cells(&self, index: usize) -> &[U32Id<BVoxPaletteCell>] {
        &self.materials[index]
    }

    /// Number of distinct materials.
    pub(crate) fn len(&self) -> usize {
        self.materials.len()
    }
}

/// Resolves the distinct merged materials `object` uses, first seen in raster
/// order. Every live voxel's cell tuple is folded into the set; two voxels share
/// a material exactly when they sample the same cell from every reference.
pub(crate) fn resolve_used_materials(object: &VoxObject) -> UsedMaterials {
    let references: Vec<(U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>)> =
        object.iter_palette_refs().collect();

    let mut materials: Vec<Vec<U32Id<BVoxPaletteCell>>> = Vec::new();
    let mut seen: HashSet<Vec<u32>> = HashSet::new();

    for voxel in object.iter_live() {
        let cells: Vec<U32Id<BVoxPaletteCell>> = references
            .iter()
            .map(|&(reference, _)| {
                object
                    .voxel_cell(voxel, reference)
                    .expect("a live voxel samples a cell from every reference")
            })
            .collect();

        let key: Vec<u32> = cells.iter().map(|cell| cell.to_u32()).collect();

        if seen.insert(key) {
            materials.push(cells);
        }
    }

    UsedMaterials {
        references,
        materials,
    }
}
