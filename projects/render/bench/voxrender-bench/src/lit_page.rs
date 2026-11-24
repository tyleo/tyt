use crate::{Error, Result};
use std::time::{Duration, Instant};
use wgpu::{
    BindGroup, Buffer, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipeline, Device,
    PollType, Queue,
};

/// The cells of a fill's first dispatch, and the most of any.
const FIRST: u32 = 1 << 10;
const MOST: u32 = 1 << 16;

/// The time a dispatch aims to take. A frame the headset draws meanwhile
/// still ends in time.
const AIM: Duration = Duration::from_millis(3);

/// The lit words of some of a scene's placements, as a
/// [`LitFill`](crate::LitFill) fills them.
pub struct LitPage {
    /// The first word of each placement's tiles' words, in the order the
    /// page took its placements.
    pub places: Vec<u32>,

    /// The first word of each placement's glass tiles' words, which
    /// follow every placement's tiles' words.
    pub glass_places: Vec<u32>,

    /// The words a fill writes:
    ///
    /// 1. A word for each record cell, as [`LitCorners`](crate::LitCorners)
    ///    lays it out. The fill writes its low 24 bits and keeps the rest,
    ///    which a caller can write beforehand
    /// 2. A word for each glass cell: the sun's light on its surfaces, 8 bits
    ///    a channel from the low bits up, under 8 zero bits
    /// 3. [`TileMasks::SPARE`](crate::TileMasks::SPARE) spare words
    /// 4. With a material that passes light, each record cell's tint in a
    ///    glass cell's layout, [`LitFill::page`](crate::LitFill::page)'s
    ///    `tints` words past the cell's word
    pub lit: Buffer,

    pub pipeline: ComputePipeline,

    pub bind_group: BindGroup,

    /// The cell a dispatch's fill ends before, then the first it fills.
    pub frame: Buffer,

    pub cells: u32,
}

impl LitPage {
    /// Fills every cell and returns once the GPU has. Each dispatch takes as
    /// many cells as would fill within [`AIM`] at the pace of the one before,
    /// because a device drops a queue of dispatches that keeps it from its
    /// frames. Errors if waiting on the GPU fails.
    pub fn fill(&self, device: &Device, queue: &Queue) -> Result<()> {
        let mut first = 0;
        let mut batch = FIRST;

        while first < self.cells {
            let count = batch.min(self.cells - first);

            // A write lands before the commands of the submit after it.
            queue.write_buffer(
                &self.frame,
                0,
                bytemuck::cast_slice(&[first + count, first]),
            );

            let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("lit fill"),
            });

            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("lit fill"),
                    timestamp_writes: None,
                });

                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.dispatch_workgroups(count.div_ceil(64), 1, 1);
            }

            let start = Instant::now();
            let submission_index = Some(queue.submit([encoder.finish()]));

            device
                .poll(PollType::Wait {
                    submission_index,
                    timeout: None,
                })
                .map_err(Error::Poll)?;

            let paced =
                f64::from(count) * AIM.as_secs_f64() / start.elapsed().as_secs_f64().max(1.0e-6);

            first += count;
            batch = (paced as u32).clamp(64, MOST);
        }

        Ok(())
    }
}
