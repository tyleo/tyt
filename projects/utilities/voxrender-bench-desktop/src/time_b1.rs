use crate::{Error, Result};
use voxrender_bench::{B1Timer, Gpu};
use wgpu::{CommandEncoderDescriptor, PollType};

/// Submits B1's timed passes over `width` by `height` eyes one at a time
/// until `windows` windows report, and returns the reports.
pub fn time_b1(gpu: &Gpu, width: u32, height: u32, windows: usize) -> Result<String> {
    let mut timer = B1Timer::new(&gpu.device, &gpu.queue, width, height).map_err(Error::Bench)?;
    let mut reports = Vec::new();

    while reports.len() < windows {
        let mut encoder = gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("b1 times"),
            });

        timer
            .record(&gpu.device, &gpu.queue, &mut encoder)
            .map_err(Error::Bench)?;
        gpu.queue.submit([encoder.finish()]);

        if let Some(report) = timer
            .collect(&gpu.device, &gpu.queue)
            .map_err(Error::Bench)?
        {
            reports.push(report);
        }

        // An unpaced loop inflates the passes' times.
        gpu.device
            .poll(PollType::wait_indefinitely())
            .map_err(Error::Poll)?;
    }

    Ok(reports.join("\n"))
}
