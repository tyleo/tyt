use serde::{Deserialize, Serialize};

/// The per-snapshot extent stat (`st.extent`). Voxel Max stores this as a
/// single-key dictionary `{o: <order>}` — a constant `5` for the 32³ chunk grid
/// (2⁵ = 32) — not as a vector, so it is modeled as a struct to match that shape
/// exactly (a 3-element array here makes Voxel Max's decoder crash).
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct VXExtent {
    /// Order tag (`o`); observed constant `5` (2⁵ = 32, the chunk edge length).
    pub o: i64,
}
