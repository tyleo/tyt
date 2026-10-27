use crate::{Error, Result};
use branded_id::U32Id;
use std::{collections::HashMap, ops::Range};
use ty_math::TyVector3U32;
use voxcore::BVoxVoxel;
use voxrender::{RenderObject, RenderScene};
use voxsurface::{SurfaceGrid, SurfaceMethod, mesh_grid};
use wgpu::{
    Buffer, BufferUsages, Device,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The greedy quads of a scene's objects, merged by occupancy alone and
/// uploaded once for every raster of the scene. Each placement draws its
/// object's quads.
pub struct RasterMesh {
    /// Four `u32`s per vertex: its corner in its object's grid, then its
    /// face's [`PixelHit`](voxrender_wgpu::PixelHit) code.
    pub vertices: Buffer,

    /// Six indices per quad, each counted from its object's first vertex.
    pub indices: Buffer,

    /// Each placement's draw in placement order: its object's range of
    /// [`indices`](Self::indices) and the object's first vertex.
    pub draws: Vec<(Range<u32>, i32)>,

    /// Each placement's grid size, one `vec4<u32>` each.
    pub bounds: Buffer,

    /// The low bits of a packed hit that hold its voxel id. The face sits
    /// above them, then the placement's index.
    pub voxel_bits: u32,
}

impl RasterMesh {
    /// The bits of a packed hit that hold its face.
    pub const FACE_BITS: u32 = 3;

    /// Meshes each object of `scene` that a placement draws. Errors unless
    /// each hit packs into 32 bits.
    pub fn new(device: &Device, scene: &RenderScene) -> Result<Self> {
        let objects: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| {
                let object = scene
                    .object(placement.object_id)
                    .expect("a placement's object is one of the scene's");

                (placement.object_id, object)
            })
            .collect();

        let largest = objects
            .iter()
            .map(|(_, object)| {
                let bounds = object.bounds();

                u64::from(bounds.x) * u64::from(bounds.y) * u64::from(bounds.z)
            })
            .max()
            .unwrap_or(0);

        let voxel_bits = u64::BITS - largest.saturating_sub(1).leading_zeros();
        let placement_bits = usize::BITS - objects.len().saturating_sub(1).leading_zeros();

        if voxel_bits + Self::FACE_BITS + placement_bits > u32::BITS {
            return Err(Error::RasterHit {
                placements: objects.len(),
                voxel_bits,
            });
        }

        let mut ranges = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut bounds = Vec::new();
        let mut draws = Vec::new();

        for (object_id, object) in objects {
            bounds.extend(object.bounds().to_array().into_iter().chain([0]));

            let draw = ranges.entry(object_id).or_insert_with(|| {
                let mesh = mesh_grid(&Occupancy(object), SurfaceMethod::Greedy);

                let first = i32::try_from(vertices.len() / 4)
                    .expect("a scene's vertices number under 2^31");

                for (quad, span) in mesh.spans.iter().enumerate() {
                    let axis = u32::try_from(span.d).expect("an axis is under 3");
                    let face = axis * 2 + u32::from(span.sign > 0);

                    for position in &mesh.positions[quad * 4..quad * 4 + 4] {
                        vertices.extend(position.as_uvec3().to_array());
                        vertices.push(face);
                    }
                }

                let start = indices.len();

                indices.extend(&mesh.indices);

                let range = [start, indices.len()]
                    .map(|end| u32::try_from(end).expect("a scene's indices number under 2^32"));

                (range[0]..range[1], first)
            });

            draws.push(draw.clone());
        }

        // A binding cannot be empty.
        if bounds.is_empty() {
            bounds.resize(4, 0);
        }

        let init = |label, words: &[u32], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents: &words
                    .iter()
                    .flat_map(|word| word.to_le_bytes())
                    .collect::<Vec<_>>(),
                usage,
            })
        };

        Ok(RasterMesh {
            vertices: init("raster vertices", &vertices, BufferUsages::VERTEX),
            indices: init("raster indices", &indices, BufferUsages::INDEX),
            draws,
            bounds: init("raster bounds", &bounds, BufferUsages::STORAGE),
            voxel_bits,
        })
    }
}

/// An object's cells read by occupancy alone. Every live cell is opaque, so
/// a greedy run merges across materials.
struct Occupancy<'a>(&'a RenderObject);

impl SurfaceGrid for Occupancy<'_> {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.0.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.0
            .voxel_id(position)
            .filter(|&voxel_id| self.0.is_live(voxel_id))
    }

    fn is_opaque(&self, _cell: Self::Cell) -> bool {
        true
    }

    fn shares_material(&self, _cell: Self::Cell, _other: Self::Cell) -> bool {
        true
    }
}
