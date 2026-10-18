use crate::{Error, Result};
use wgpu::{
    AdapterInfo, Device, DeviceDescriptor, Instance, PowerPreference, Queue, RequestAdapterOptions,
};

/// A headless GPU device.
pub struct Gpu {
    /// The adapter the device runs on.
    pub info: AdapterInfo,

    /// The device.
    pub device: Device,

    /// The device's queue.
    pub queue: Queue,
}

impl Gpu {
    /// The high-performance adapter's device with default limits.
    pub fn new() -> Result<Self> {
        let instance = Instance::default();

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            ..RequestAdapterOptions::default()
        }))
        .map_err(Error::Adapter)?;

        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("voxrender-bench"),
            ..DeviceDescriptor::default()
        }))
        .map_err(Error::Device)?;

        Ok(Gpu {
            info: adapter.get_info(),
            device,
            queue,
        })
    }
}
