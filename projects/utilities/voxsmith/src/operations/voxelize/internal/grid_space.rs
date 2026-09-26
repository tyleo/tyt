use ty_math::{TyBoundsF64, TyVector3F64, TyVector3U32};

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

    /// The grid of `size` voxels covering `bounds` from its min corner.
    pub fn fit(bounds: &TyBoundsF64, size: TyVector3F64) -> Self {
        let extent = bounds.size();

        let counts = TyVector3U32::new(
            cell_count(extent.x, size.x),
            cell_count(extent.y, size.y),
            cell_count(extent.z, size.z),
        );

        Self::new(bounds.min(), size, counts)
    }

    /// The voxel edge length on each axis.
    pub fn size(&self) -> TyVector3F64 {
        self.size
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

/// The cells of `size` covering `extent`, at least one.
fn cell_count(extent: f64, size: f64) -> u32 {
    let cells = extent / size;
    let whole = cells.round();

    let count = if (cells - whole).abs() <= 1e-9 * whole.max(1.0) {
        whole
    } else {
        cells.ceil()
    };

    count.max(1.0) as u32
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
    use ty_math::{TyBoundsF64, TyVector3F64, TyVector3U32};

    #[test]
    fn fit_covers_the_extent_and_rounds_a_whole_count_through_float_error() {
        // `3.0 / 0.1` is a hair over 30 in floating point.
        let bounds = TyBoundsF64::from_min_size(
            TyVector3F64::new(1.0, 0.0, 0.0),
            TyVector3F64::new(3.0, 0.25, 0.0),
        );

        let space = GridSpace::fit(&bounds, TyVector3F64::splat(0.1));

        assert_eq!(space.counts(), TyVector3U32::new(30, 3, 1));
        assert_eq!(space.to_grid(TyVector3F64::new(4.0, 0.0, 0.0))[0], 30.0);
    }
}
