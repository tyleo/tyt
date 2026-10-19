use crate::{Error, Result};
use voxrender_bench::Gpu;
use wgpu::{DeviceDescriptor, Instance, PowerPreference, RequestAdapterOptions};

/// The high-performance adapter's device with default limits.
pub fn headless_gpu() -> Result<Gpu> {
    let instance = Instance::default();

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        ..RequestAdapterOptions::default()
    }))
    .map_err(Error::Adapter)?;

    let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
        label: Some("voxrender-bench-desktop"),
        ..DeviceDescriptor::default()
    }))
    .map_err(Error::Device)?;

    Ok(Gpu {
        info: adapter.get_info(),
        device,
        queue,
    })
}
