use crate::{Error, Result};
use std::{collections::BTreeMap, fs, mem, time::Instant};

/// The kgsl file that holds the GPU's current clock in Hz.
const CLOCK_PATH: &str = "/sys/class/kgsl/kgsl-3d0/devfreq/cur_freq";

/// Each GPU clock's share of a stretch of time.
pub struct GpuClock {
    // The seconds at each clock in Hz since the last `take`.
    seconds: BTreeMap<u32, f64>,

    // When the last sample read the clock.
    sampled: Instant,
}

impl GpuClock {
    /// Starts the count now. Errors unless the app can read the clock.
    pub fn new() -> Result<Self> {
        read_clock()?;

        Ok(GpuClock {
            seconds: BTreeMap::new(),
            sampled: Instant::now(),
        })
    }

    /// Counts the time since the last sample at the current clock.
    pub fn sample(&mut self) -> Result<()> {
        let clock = read_clock()?;
        let now = Instant::now();

        *self.seconds.entry(clock).or_default() += (now - self.sampled).as_secs_f64();
        self.sampled = now;

        Ok(())
    }

    /// A report line of each clock's share of the time since the last call,
    /// fastest clock first. Starts a new count.
    pub fn take(&mut self) -> String {
        let seconds = mem::take(&mut self.seconds);
        let total: f64 = seconds.values().sum();

        assert!(total > 0.0, "a frame samples the clock");

        let shares: Vec<_> = seconds
            .iter()
            .rev()
            .map(|(&hz, &time)| {
                format!(
                    "{:.0} MHz {:.0}%",
                    f64::from(hz) / 1e6,
                    100.0 * time / total
                )
            })
            .collect();

        format!("GPU clock: {}", shares.join(", "))
    }
}

/// The GPU's current clock in Hz.
fn read_clock() -> Result<u32> {
    let text = fs::read_to_string(CLOCK_PATH).map_err(Error::ClockRead)?;

    text.trim()
        .parse()
        .map_err(|_| Error::ClockParse(text.trim().to_owned()))
}
