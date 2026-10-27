use crate::{SurfaceGrid, SurfaceMesh, SurfaceMethod, mesh_grid_keyed};

/// Meshes the boundary of `grid`'s solid cells by `method`. A greedy run
/// merges across every cell.
pub fn mesh_grid<G: SurfaceGrid>(grid: &G, method: SurfaceMethod) -> SurfaceMesh<G::Cell> {
    mesh_grid_keyed(grid, method, &|_| 0, &|_| true)
}
