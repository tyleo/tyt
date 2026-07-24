use crate::{MeshTriangle, triangle_bounds};
use ty_math::{TyVector3F64, TyVector3U32};

/// The affine map from world space onto the voxel grid: the grid's min corner
/// and per-axis voxel edge length. The rasterizer and the texel sampler both
/// build it from the same bounds, so a sampled surface point lands in the same
/// cell the occupancy grid filled.
pub(crate) struct GridSpace {
    min: TyVector3F64,
    size: TyVector3F64,
    counts: [usize; 3],
}

impl GridSpace {
    /// The grid space for `counts` fit tightly around `triangles`, or `None` when
    /// the soup has no points.
    pub fn from_triangles(triangles: &[MeshTriangle], counts: TyVector3U32) -> Option<Self> {
        let points = triangles.iter().flat_map(|triangle| triangle.points);
        let (min, max) = triangle_bounds(points)?;
        Some(Self::from_bounds(min, max, counts))
    }

    /// The grid space for `counts` over the box `[min, max]`.
    pub fn from_bounds(min: TyVector3F64, max: TyVector3F64, counts: TyVector3U32) -> Self {
        let counts = [counts.x as usize, counts.y as usize, counts.z as usize];
        let size = TyVector3F64::new(
            voxel_size(min.x, max.x, counts[0]),
            voxel_size(min.y, max.y, counts[1]),
            voxel_size(min.z, max.z, counts[2]),
        );
        Self { min, size, counts }
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
        let [nx, ny, nz] = self.counts;
        let x = clamp_index(grid[0], nx.saturating_sub(1));
        let y = clamp_index(grid[1], ny.saturating_sub(1));
        let z = clamp_index(grid[2], nz.saturating_sub(1));
        x * ny * nz + y * nz + z
    }

    /// The world-space center of the cell at raster index `cell`.
    pub fn cell_center(&self, cell: usize) -> TyVector3F64 {
        let [_, ny, nz] = self.counts;
        let plane = ny * nz;
        let (x, remainder) = (cell / plane, cell % plane);
        let (y, z) = (remainder / nz, remainder % nz);
        let offset = TyVector3F64::new(x as f64 + 0.5, y as f64 + 0.5, z as f64 + 0.5);
        self.min + offset * self.size
    }
}

/// The edge length of one voxel on an axis, or `1.0` for a zero-extent axis.
fn voxel_size(min: f64, max: f64, count: usize) -> f64 {
    let extent = max - min;
    if extent > 0.0 && count > 0 {
        extent / count as f64
    } else {
        1.0
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
