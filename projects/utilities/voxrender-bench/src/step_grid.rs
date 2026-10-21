use wgpu::{
    Buffer, BufferUsages, Device, Extent3d, Queue, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    util::{BufferInitDescriptor, DeviceExt, TextureDataOrder},
};

/// The cells along each axis of B2's grid.
pub const STEP_GRID: u32 = 256;

/// The cells along each axis of a mask's brick.
const BRICK: u32 = 4;

/// B2's grid in each layout a step loop reads. Every layout holds the same
/// scrambled cells, which no compression shrinks.
pub struct StepGrid {
    /// The cells in raster order, z fastest.
    pub cells: Buffer,

    /// The cells as a 3D texture whose x is the grid's z.
    pub volume: TextureView,

    /// The cells as a 2D array with one layer per x, whose texel x is the
    /// grid's z.
    pub layers: TextureView,

    /// One u32 pair per brick in raster order.
    pub masks: Buffer,
}

impl StepGrid {
    /// Uploads the grid to `device`.
    pub fn new(device: &Device, queue: &Queue) -> Self {
        let words = |count: u32| -> Vec<u8> {
            (0..count)
                .flat_map(|index| scramble(index).to_le_bytes())
                .collect()
        };

        let bytes = words(STEP_GRID.pow(3));
        let size = Extent3d {
            width: STEP_GRID,
            height: STEP_GRID,
            depth_or_array_layers: STEP_GRID,
        };

        let texture = |label, dimension, view_dimension| {
            device
                .create_texture_with_data(
                    queue,
                    &TextureDescriptor {
                        label: Some(label),
                        size,
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension,
                        format: TextureFormat::R32Uint,
                        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                    TextureDataOrder::LayerMajor,
                    &bytes,
                )
                .create_view(&TextureViewDescriptor {
                    dimension: Some(view_dimension),
                    ..TextureViewDescriptor::default()
                })
        };

        let volume = texture(
            "step volume",
            TextureDimension::D3,
            TextureViewDimension::D3,
        );
        let layers = texture(
            "step layers",
            TextureDimension::D2,
            TextureViewDimension::D2Array,
        );

        let cells = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("step cells"),
            contents: &bytes,
            usage: BufferUsages::STORAGE,
        });

        let masks = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("step masks"),
            contents: &words((STEP_GRID / BRICK).pow(3) * 2),
            usage: BufferUsages::STORAGE,
        });

        StepGrid {
            cells,
            volume,
            layers,
            masks,
        }
    }
}

/// A hash of `index` below 2^31.
fn scramble(index: u32) -> u32 {
    let mut x = index;

    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;

    x >> 1
}
