use crate::{Error, KernelWork, Result, b2_kernels};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use wgpu::{CommandEncoder, Device, PollType, QuerySet, QuerySetDescriptor, QueryType, Queue};

/// The B2 kernels a profile runs, each with the passes a batch records. A
/// batch takes about 10 ms on Quest 3.
const PROFILED: [(&str, u32); 6] = [
    ("tree walk, S1 art room", 1),
    ("tree walk, S5 grazing floor", 1),
    ("ray query, float march, 16^3 bricks, S1 art room", 1),
    ("buffer, start hoisted, 16 steps", 2),
    ("compute launch, 0 steps", 5),
    ("fragment launch", 50),
];

/// How long each kernel runs before the next.
const STRETCH: Duration = Duration::from_secs(10);

/// The batches that can be in flight at once.
const IN_FLIGHT: usize = 2;

/// B2's kernels run one at a time for a stretch each, in turn, so a
/// sampling profiler sees one kernel at once.
pub struct KernelProfile {
    kernels: Vec<Profiled>,

    // The batches the current kernel's stretch has recorded.
    batches: u32,

    // Timestamps the passes write and no one reads.
    query_set: QuerySet,

    current: usize,

    // When the current kernel's stretch began, from its first batch.
    started: Option<Instant>,

    // Whether `record` added a batch since the last `submitted`.
    recorded: bool,

    in_flight: Arc<AtomicUsize>,
}

/// A profiled kernel and the passes its batch records.
struct Profiled {
    label: String,

    work: KernelWork,

    passes: u32,
}

impl KernelProfile {
    /// Builds the profiled kernels for a `width` by `height` eye image. With
    /// `only`, the profile runs the kernels whose labels hold that text, and
    /// each of their batches records one pass. Errors unless a kernel runs.
    pub fn new(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        only: Option<&str>,
    ) -> Result<Self> {
        let mut kernels = b2_kernels(device, queue, width, height, true)?.kernels;

        let kernels: Vec<_> = match only {
            Some(text) => kernels
                .into_iter()
                .filter(|(label, _)| label.contains(text))
                .map(|(label, work)| Profiled {
                    label,
                    work,
                    passes: 1,
                })
                .collect(),

            None => PROFILED
                .iter()
                .map(|&(label, passes)| {
                    let at = kernels
                        .iter()
                        .position(|(kernel, _)| kernel == label)
                        .expect("B2 has every profiled kernel");
                    let (label, work) = kernels.swap_remove(at);

                    Profiled {
                        label,
                        work,
                        passes,
                    }
                })
                .collect(),
        };

        if let (Some(text), true) = (only, kernels.is_empty()) {
            return Err(Error::NoKernel(text.to_owned()));
        }

        let query_set = device.create_query_set(&QuerySetDescriptor {
            label: Some("profile timestamps"),
            ty: QueryType::Timestamp,
            count: 2,
        });

        Ok(KernelProfile {
            kernels,
            batches: 0,
            query_set,
            current: 0,
            started: None,
            recorded: false,
            in_flight: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Records the current kernel's next batch into `encoder`, unless
    /// enough batches are in flight. When the batch starts a stretch,
    /// returns a line that holds the kernel's label, after the batches the
    /// last stretch recorded and its seconds.
    pub fn record(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) -> Result<Option<String>> {
        assert!(
            !self.recorded,
            "`submitted` follows each submit of a recorded batch"
        );

        device.poll(PollType::Poll).map_err(Error::Poll)?;

        if self.in_flight.load(Ordering::Acquire) >= IN_FLIGHT {
            return Ok(None);
        }

        let mut line = None;

        match self.started {
            Some(started) if started.elapsed() < STRETCH => {}

            Some(started) => {
                line = Some(format!(
                    "{} batches in {:.2} s. ",
                    self.batches,
                    started.elapsed().as_secs_f64()
                ));

                self.current = (self.current + 1) % self.kernels.len();
                self.started = None;
            }

            None => {}
        }

        let Profiled {
            label,
            work,
            passes,
        } = &self.kernels[self.current];

        if self.started.is_none() {
            self.started = Some(Instant::now());
            self.batches = 0;

            line = Some(format!(
                "{}{label}, {passes} passes a batch",
                line.unwrap_or_default()
            ));
        }

        for _ in 0..*passes {
            work.record(encoder, label, &self.query_set, 0);
        }

        self.batches += 1;
        self.recorded = true;

        Ok(line)
    }

    /// Call after submitting the encoder `record` wrote into.
    pub fn submitted(&mut self, queue: &Queue) {
        if !self.recorded {
            return;
        }

        self.recorded = false;
        self.in_flight.fetch_add(1, Ordering::AcqRel);

        let in_flight = Arc::clone(&self.in_flight);

        queue.on_submitted_work_done(move || {
            in_flight.fetch_sub(1, Ordering::AcqRel);
        });
    }
}
