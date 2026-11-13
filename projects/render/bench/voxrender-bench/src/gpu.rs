use wgpu::{AdapterInfo, Device, Queue};

/// The GPU device a view draws on.
pub struct Gpu {
    /// The adapter the device runs on.
    pub info: AdapterInfo,

    /// The device.
    pub device: Device,

    /// The device's queue.
    pub queue: Queue,
}
