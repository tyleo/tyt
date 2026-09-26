use ty_math::{TyBoundsF64, TyVector3F64, TyVector3I32, TyVector3U32};

/// The map from world space onto the voxel grid. The rasterizer and the
/// sampler share one so both agree on which cell a point falls in.
pub(crate) struct GridSpace {
    min: TyVector3F64,
    size: TyVector3F64,
    counts: TyVector3U32,
}

impl GridSpace {
    /// A grid from its corner, voxel size, and counts.
    pub fn new(min: TyVector3F64, size: TyVector3F64, counts: TyVector3U32) -> Self {
        Self { min, size, counts }
    }

    /// The grid of `size` voxels covering `bounds` on the lattice of `size`
    /// cells anchored at the origin, with at least one cell per axis.
    pub fn on_lattice(bounds: &TyBoundsF64, size: TyVector3F64) -> Self {
        let min = bounds.min().to_array();
        let max = bounds.max().to_array();
        let size = size.to_array();

        let first = [0, 1, 2].map(|axis| snap(min[axis] / size[axis]).floor());
        let last = [0, 1, 2].map(|axis| snap(max[axis] / size[axis]).ceil());
        let counts = [0, 1, 2].map(|axis| (last[axis] - first[axis]).max(1.0) as u32);

        Self::new(
            TyVector3F64::new(first[0] * size[0], first[1] * size[1], first[2] * size[2]),
            TyVector3F64::new(size[0], size[1], size[2]),
            TyVector3U32::new(counts[0], counts[1], counts[2]),
        )
    }

    /// The lattice cell at the grid's min corner, in voxels from the origin.
    /// Whole for a grid built [`on_lattice`](Self::on_lattice).
    pub fn min_cell(&self) -> TyVector3I32 {
        let [x, y, z] = (self.min / self.size)
            .to_array()
            .map(|cell| cell.round() as i32);
        TyVector3I32::new(x, y, z)
    }

    /// The cell counts per axis.
    pub fn counts(&self) -> TyVector3U32 {
        self.counts
    }

    /// `point` in grid coordinates, where one unit is one voxel from the min
    /// corner.
    pub fn to_grid(&self, point: TyVector3F64) -> [f64; 3] {
        ((point - self.min) / self.size).to_array()
    }

    /// The raster cell index (`x*Y*Z + y*Z + z`) `point` falls in, floored and
    /// clamped to the grid.
    pub fn cell_index(&self, point: TyVector3F64) -> usize {
        let grid = self.to_grid(point);
        let [nx, ny, nz] = self.counts.to_array().map(|count| count as usize);
        let x = clamp_index(grid[0], nx.saturating_sub(1));
        let y = clamp_index(grid[1], ny.saturating_sub(1));
        let z = clamp_index(grid[2], nz.saturating_sub(1));
        x * ny * nz + y * nz + z
    }

    /// The world-space center of the cell at raster index `cell`.
    pub fn cell_center(&self, cell: usize) -> TyVector3F64 {
        let [_, ny, nz] = self.counts.to_array().map(|count| count as usize);
        let plane = ny * nz;
        let (x, remainder) = (cell / plane, cell % plane);
        let (y, z) = (remainder / nz, remainder % nz);
        let offset = TyVector3F64::new(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);
        self.min + offset * self.size
    }
}

/// `cells`, or the whole number it lies within float error of.
fn snap(cells: f64) -> f64 {
    let whole = cells.round();

    if (cells - whole).abs() <= 1e-9 * whole.abs().max(1.0) {
        whole
    } else {
        cells
    }
}

/// A floored grid coordinate clamped to `0..=last`.
pub(crate) fn clamp_index(value: f64, last: usize) -> usize {
    if value < 0.0 {
        0
    } else {
        (value as usize).min(last)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::voxelize::GridSpace;
    use ty_math::{TyBoundsF64, TyVector3F64, TyVector3I32, TyVector3U32};

    #[test]
    fn the_lattice_covers_the_extent_and_rounds_a_whole_count_through_float_error() {
        // `3.0 / 0.1` is a hair over 30 in floating point.
        let bounds = TyBoundsF64::from_min_size(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(3.0, 0.25, 0.0),
        );

        let space = GridSpace::on_lattice(&bounds, TyVector3F64::splat(0.1));

        assert_eq!(space.counts(), TyVector3U32::new(30, 3, 1));
        assert_eq!(space.min_cell(), TyVector3I32::new(10, 0, 0));
        assert_eq!(space.to_grid(TyVector3F64::new(4.0, 0.0, 0.0))[0], 30.0);
    }

    #[test]
    fn the_lattice_snaps_the_min_corner_below_the_bounds() {
        let bounds = TyBoundsF64::from_min_size(
            TyVector3F64::new(-0.5, 0.25, 2.0),
            TyVector3F64::new(2.0, 1.0, 0.0),
        );

        let space = GridSpace::on_lattice(&bounds, TyVector3F64::splat(1.0));

        assert_eq!(space.min_cell(), TyVector3I32::new(-1, 0, 2));
        assert_eq!(space.counts(), TyVector3U32::new(3, 2, 1));
    }
}
