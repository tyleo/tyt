use crate::{Error, Result};
use voxrender_bench::{Gpu, PassTimer, TimedBench};
use wgpu::{CommandEncoderDescriptor, PollType};

/// Submits `bench`'s timed passes over `width` by `height` eyes one at a
/// time until `windows` windows report, and returns the reports. When
/// `capture`, a graphics debugger then captures one pass of each kernel. The
/// captured passes join no report.
pub fn time_passes(
    gpu: &Gpu,
    bench: TimedBench,
    width: u32,
    height: u32,
    windows: usize,
    capture: bool,
) -> Result<String> {
    let mut timer =
        PassTimer::new(&gpu.device, &gpu.queue, bench, width, height).map_err(Error::Bench)?;
    let mut reports = Vec::new();

    while reports.len() < windows {
        if let Some(report) = run_pass(gpu, &mut timer)? {
            reports.push(report);
        }
    }

    if capture {
        // SAFETY: the bench records no work outside wgpu while capturing.
        unsafe { gpu.device.start_graphics_debugger_capture() };

        // A pass can skip its turn while the last one runs, so the capture
        // waits on each.
        for _ in 0..timer.kernel_count() {
            run_pass(gpu, &mut timer)?;
        }

        // SAFETY: the capture started above, and the GPU has finished its
        // passes.
        unsafe { gpu.device.stop_graphics_debugger_capture() };
    }

    Ok(reports.join("\n"))
}

/// Records and submits the timer's next pass, waits on the GPU, and returns
/// a window's report if one completed.
fn run_pass(gpu: &Gpu, timer: &mut PassTimer) -> Result<Option<String>> {
    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("timed passes"),
        });

    timer
        .record(&gpu.device, &gpu.queue, &mut encoder)
        .map_err(Error::Bench)?;
    gpu.queue.submit([encoder.finish()]);

    let report = timer
        .collect(&gpu.device, &gpu.queue)
        .map_err(Error::Bench)?;

    // An unpaced loop inflates the passes' times.
    gpu.device
        .poll(PollType::wait_indefinitely())
        .map_err(Error::Poll)?;

    Ok(report)
}
