use crate::{Error, Result, XrGpu};
use voxrender_bench::{Gpu, run_b1};

/// The side of each view's square image, the size of B1's desktop run.
const SIZE: u32 = 256;

/// Runs B1 on the session's device and logs its report one line at a time.
pub fn log_b1(gpu: &XrGpu) -> Result<()> {
    let gpu = Gpu {
        info: gpu.adapter.get_info(),
        device: gpu.device.clone(),
        queue: gpu.queue.clone(),
    };

    let report = run_b1(&gpu, SIZE).map_err(Error::Bench)?;

    for line in report.lines() {
        log::info!("b1 {line}");
    }

    Ok(())
}
