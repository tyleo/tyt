use crate::{Error, Result};
use voxrender_bench::Gpu;
use wgpu::{DeviceDescriptor, Features, Instance, PowerPreference, RequestAdapterOptions};

/// The high-performance adapter's device with default limits, and with
/// timestamp queries when the adapter has them.
pub fn headless_gpu() -> Result<Gpu> {
    let instance = Instance::default();

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        ..RequestAdapterOptions::default()
    }))
    .map_err(Error::Adapter)?;

    let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
        label: Some("voxrender-bench-desktop"),
        required_features: adapter.features() & Features::TIMESTAMP_QUERY,
        ..DeviceDescriptor::default()
    }))
    .map_err(Error::Device)?;

    Ok(Gpu {
        info: adapter.get_info(),
        device,
        queue,
    })
}
