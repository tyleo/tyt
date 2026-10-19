use crate::{ChainKernel, Error, Result, Walk, far_wall_scene};
use std::{
    result::Result as StdResult,
    sync::mpsc::{self, Receiver, Sender},
};
use voxrender_wgpu::HitTrace;
use wgpu::{
    Buffer, BufferAsyncError, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, ComputePassTimestampWrites,
    Device, Features, MapMode, PollType, QUERY_RESOLVE_BUFFER_ALIGNMENT, QuerySet,
    QuerySetDescriptor, QueryType, Queue,
};

/// The far wall's depths in cells. The walks' growth between them is the
/// steps' cost alone.
const DEPTHS: [u32; 2] = [8, 16];

/// The passes before the timer keeps samples.
const WARM_UP: u32 = 300;

/// Each kernel's samples in a window.
const MEASURED: usize = 600;

/// The passes whose timestamps can be in flight at once.
const SLOTS: u32 = 8;

/// The bytes of one pass's two timestamps.
const SLOT_BYTES: u64 = 16;

/// B1's timing over both eyes, one kernel per pass.
pub struct B1Timer {
    kernels: Vec<Kernel>,

    query_set: QuerySet,

    resolved: Buffer,

    slots: Vec<Slot>,

    // The slot of the pass `record` added last, until `collect` sees it
    // submitted.
    recorded: Option<usize>,

    // Whether a submitted pass has yet to finish.
    running: bool,

    sender: Sender<Progress>,

    receiver: Receiver<Progress>,

    // Nanoseconds per timestamp tick.
    period: f64,

    width: u32,

    height: u32,

    passes: u32,

    windows: u32,
}

/// What one timed pass runs.
enum Work {
    Walk(HitTrace),

    Chain(ChainKernel),
}

/// A kernel and its window's samples in milliseconds.
struct Kernel {
    label: String,

    work: Work,

    samples: Vec<f64>,
}

/// One pass's timestamp readback.
struct Slot {
    buffer: Buffer,

    // The pass's kernel while its timestamps are in flight, and whether the
    // pass is past the warm-up.
    pending: Option<(usize, bool)>,
}

/// A slot's progress, which the GPU's callbacks send.
enum Progress {
    /// The slot's pass has finished.
    Done(usize),

    /// The slot's readback has mapped, or failed to.
    Mapped(usize, StdResult<(), BufferAsyncError>),
}

impl B1Timer {
    /// Builds the kernels for a `width` by `height` eye image. Errors unless
    /// the device has timestamp queries.
    pub fn new(device: &Device, queue: &Queue, width: u32, height: u32) -> Result<Self> {
        if !device.features().contains(Features::TIMESTAMP_QUERY) {
            return Err(Error::NoTimestampQuery);
        }

        let mut kernels = Vec::new();

        for depth in DEPTHS {
            let scene = far_wall_scene(depth);
            let (view_id, _) = scene
                .iter_views()
                .next()
                .expect("the far wall scene has a view");

            for (name, walk) in [("integer", Walk::Integer), ("float", Walk::Float)] {
                let trace = walk.trace(device, &scene, view_id, width, height)?;

                kernels.push((format!("{name} walk, {depth} deep"), Work::Walk(trace)));
            }
        }

        for (name, float) in [("i32", false), ("f32", true)] {
            kernels.push((
                format!("{name} chain"),
                Work::Chain(ChainKernel::new(device, width, height, float)),
            ));
        }

        let kernels = kernels
            .into_iter()
            .map(|(label, work)| Kernel {
                label,
                work,
                samples: Vec::with_capacity(MEASURED),
            })
            .collect();

        let query_set = device.create_query_set(&QuerySetDescriptor {
            label: Some("b1 timestamps"),
            ty: QueryType::Timestamp,
            count: SLOTS * 2,
        });

        let resolved = device.create_buffer(&BufferDescriptor {
            label: Some("b1 resolved timestamps"),
            size: u64::from(SLOTS) * QUERY_RESOLVE_BUFFER_ALIGNMENT,
            usage: BufferUsages::QUERY_RESOLVE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let slots = (0..SLOTS)
            .map(|_| Slot {
                buffer: device.create_buffer(&BufferDescriptor {
                    label: Some("b1 timestamp readback"),
                    size: SLOT_BYTES,
                    usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }),
                pending: None,
            })
            .collect();

        let (sender, receiver) = mpsc::channel();

        Ok(B1Timer {
            kernels,
            query_set,
            resolved,
            slots,
            recorded: None,
            running: false,
            sender,
            receiver,
            period: f64::from(queue.get_timestamp_period()),
            width,
            height,
            passes: 0,
            windows: 0,
        })
    }

    /// Records the next kernel's timed pass over both eyes into `encoder`.
    /// Records nothing while the previous pass runs, so passes never overlap.
    /// Waits on the GPU while the next slot's readback is in flight.
    pub fn record(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
    ) -> Result<()> {
        assert!(
            self.recorded.is_none(),
            "`collect` follows each submit of a recorded pass"
        );

        device.poll(PollType::Poll).map_err(Error::Poll)?;

        self.advance(device, queue)?;

        if self.running {
            return Ok(());
        }

        let index = self.passes % SLOTS;
        let slot = usize::try_from(index).expect("slots fit in usize");

        // A slot's readback starts only once its pass finishes, so freeing
        // the slot can take two waits.
        while self.slots[slot].pending.is_some() {
            device
                .poll(PollType::wait_indefinitely())
                .map_err(Error::Poll)?;

            self.advance(device, queue)?;
        }

        let kernel =
            usize::try_from(self.passes).expect("pass counts fit in usize") % self.kernels.len();
        let first = index * 2;

        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some(&self.kernels[kernel].label),
                timestamp_writes: Some(ComputePassTimestampWrites {
                    query_set: &self.query_set,
                    beginning_of_pass_write_index: Some(first),
                    end_of_pass_write_index: Some(first + 1),
                }),
            });

            for _ in 0..2 {
                dispatch(&self.kernels[kernel].work, &mut pass);
            }
        }

        self.slots[slot].pending = Some((kernel, self.passes >= WARM_UP));
        self.recorded = Some(slot);
        self.passes += 1;

        Ok(())
    }

    /// Call after submitting the encoder `record` wrote into. Returns a
    /// window's report once every kernel fills one.
    pub fn collect(&mut self, device: &Device, queue: &Queue) -> Result<Option<String>> {
        if let Some(slot) = self.recorded.take() {
            self.running = true;

            let sender = self.sender.clone();

            // A callback can run after the timer drops, with no one to tell.
            queue.on_submitted_work_done(move || {
                let _ = sender.send(Progress::Done(slot));
            });
        }

        device.poll(PollType::Poll).map_err(Error::Poll)?;

        self.advance(device, queue)?;

        if self
            .kernels
            .iter()
            .any(|kernel| kernel.samples.len() < MEASURED)
        {
            return Ok(None);
        }

        let mut lines = vec![
            format!(
                "B1 times, window {}: {MEASURED} passes per kernel over two {} by {} eyes",
                self.windows, self.width, self.height
            ),
            String::new(),
            "| Kernel | p50 ms | p99 ms |".to_owned(),
            "| --- | ---: | ---: |".to_owned(),
        ];

        let mut medians = Vec::new();

        for kernel in &mut self.kernels {
            kernel.samples.sort_by(f64::total_cmp);

            let median = percentile(&kernel.samples, 50);

            lines.push(format!(
                "| {} | {median:.3} | {:.3} |",
                kernel.label,
                percentile(&kernel.samples, 99)
            ));

            medians.push(median);
            kernel.samples.clear();
        }

        let [near, far] = DEPTHS;

        lines.push(String::new());
        lines.push(format!(
            "Integer step over float step, from the walks' growth between {near} and {far} \
             cells deep: {:.3}x. i32 chain over f32 chain: {:.3}x.",
            (medians[2] - medians[0]) / (medians[3] - medians[1]),
            medians[4] / medians[5]
        ));
        lines.push(String::new());

        self.windows += 1;

        Ok(Some(lines.join("\n")))
    }

    /// Moves each slot whose pass or readback has finished to its next
    /// stage.
    fn advance(&mut self, device: &Device, queue: &Queue) -> Result<()> {
        while let Ok(progress) = self.receiver.try_recv() {
            match progress {
                Progress::Done(slot) => {
                    self.running = false;
                    self.read_back(device, queue, slot);
                }

                Progress::Mapped(slot, result) => {
                    result.map_err(Error::Map)?;
                    self.take_sample(slot)?;
                }
            }
        }

        Ok(())
    }

    fn read_back(&self, device: &Device, queue: &Queue, slot: usize) {
        let index = u32::try_from(slot).expect("slots fit in u32");
        let offset = u64::from(index) * QUERY_RESOLVE_BUFFER_ALIGNMENT;

        // Metal shows a pass's timestamps to a resolve only once the pass's
        // command buffer completes.
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("b1 timestamps"),
        });

        encoder.resolve_query_set(
            &self.query_set,
            index * 2..index * 2 + 2,
            &self.resolved,
            offset,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolved,
            offset,
            &self.slots[slot].buffer,
            0,
            SLOT_BYTES,
        );
        queue.submit([encoder.finish()]);

        let sender = self.sender.clone();

        self.slots[slot]
            .buffer
            .map_async(MapMode::Read, .., move |result| {
                let _ = sender.send(Progress::Mapped(slot, result));
            });
    }

    fn take_sample(&mut self, slot: usize) -> Result<()> {
        let buffer = &self.slots[slot].buffer;

        let ticks = {
            let view = buffer.get_mapped_range(..).map_err(Error::MapRange)?;
            let stamp = |at: usize| {
                u64::from_le_bytes(
                    view[at..at + 8]
                        .try_into()
                        .expect("a timestamp holds eight bytes"),
                )
            };

            stamp(8)
                .checked_sub(stamp(0))
                .ok_or(Error::TimestampOrder)?
        };

        buffer.unmap();

        let (kernel, counts) = self.slots[slot]
            .pending
            .take()
            .expect("a mapped slot is in flight");
        let samples = &mut self.kernels[kernel].samples;

        if counts && samples.len() < MEASURED {
            samples.push(ticks as f64 * self.period / 1e6);
        }

        Ok(())
    }
}

/// Records `work` over one eye into `pass`.
fn dispatch(work: &Work, pass: &mut ComputePass<'_>) {
    match work {
        Work::Walk(trace) => trace.dispatch(pass),
        Work::Chain(chain) => chain.dispatch(pass),
    }
}

/// The nearest-rank `percent` percentile of the sorted `samples`.
fn percentile(samples: &[f64], percent: usize) -> f64 {
    samples[(samples.len() * percent).div_ceil(100) - 1]
}
