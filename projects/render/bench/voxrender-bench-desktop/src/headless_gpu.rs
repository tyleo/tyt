use crate::{Error, Result};
use voxrender_bench::Gpu;
use wgpu::{
    DeviceDescriptor, ExperimentalFeatures, Features, Instance, Limits, PowerPreference,
    RequestAdapterOptions,
};

/// The high-performance adapter's device with default limits, and with ray
/// queries, multiview, and dual-source blending when the adapter has them.
pub fn headless_gpu() -> Result<Gpu> {
    let instance = Instance::default();

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: PowerPreference::HighPerformance,
        ..RequestAdapterOptions::default()
    }))
    .map_err(Error::Adapter)?;

    let features = adapter.features()
        & (Features::EXPERIMENTAL_RAY_QUERY | Features::MULTIVIEW | Features::DUAL_SOURCE_BLENDING);

    let mut limits = if features.contains(Features::EXPERIMENTAL_RAY_QUERY) {
        Limits::default().using_acceleration_structure_values(adapter.limits())
    } else {
        Limits::default()
    };

    // A view draws both eyes in multiview passes.
    if features.contains(Features::MULTIVIEW) {
        limits.max_multiview_view_count = 2;
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
        label: Some("voxrender-bench-desktop"),
        required_features: features,
        required_limits: limits,
        // SAFETY: the bench accepts ray queries' experimental status.
        // `view-check` judges what they draw against the reference.
        experimental_features: unsafe { ExperimentalFeatures::enabled() },
        ..DeviceDescriptor::default()
    }))
    .map_err(Error::Device)?;

    Ok(Gpu {
        info: adapter.get_info(),
        device,
        queue,
    })
}
