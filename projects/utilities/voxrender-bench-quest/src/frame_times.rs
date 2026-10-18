use crate::{Error, Result, XrSystem, check_xr};
use openxr::{
    Path, Session, Vulkan as XrVulkan,
    raw::PerformanceMetricsMETA,
    sys::{
        self, PerformanceMetricsCounterFlagsMETA, PerformanceMetricsCounterMETA,
        PerformanceMetricsCounterUnitMETA, PerformanceMetricsStateMETA,
    },
};
use std::{process::Command, ptr, result::Result as StdResult};

/// The rendered frames before sampling starts.
const WARM_UP: u32 = 300;

/// The samples each counter reports over.
const MEASURED: usize = 600;

/// The system property that pins the GPU level, which the README's run
/// steps set.
const GPU_LEVEL_PROPERTY: &str = "debug.oculus.gpuLevel";

/// The counters B0 samples, each with its report label.
const COUNTERS: [(&str, &str); 2] = [
    ("app gpu", "/perfmetrics_meta/app/gpu_frametime"),
    (
        "compositor gpu",
        "/perfmetrics_meta/compositor/gpu_frametime",
    ),
];

/// B0's app and compositor GPU frame times from XR_META_performance_metrics.
pub struct FrameTimes {
    metrics: PerformanceMetricsMETA,

    counters: Vec<Counter>,

    frames: u32,
}

/// One sampled counter.
struct Counter {
    label: &'static str,

    path: Path,

    unit: Option<PerformanceMetricsCounterUnitMETA>,

    samples: Vec<f64>,
}

impl FrameTimes {
    /// Turns on the runtime's performance metrics. None when the runtime
    /// lacks XR_META_performance_metrics.
    pub fn new(xr: &XrSystem, session: &Session<XrVulkan>) -> Result<Option<Self>> {
        let output = Command::new("getprop")
            .arg(GPU_LEVEL_PROPERTY)
            .output()
            .map_err(Error::Getprop)?;

        let level = String::from_utf8_lossy(&output.stdout);
        let level = level.trim();

        log::info!(
            "b0 gpu level override: {}",
            if level.is_empty() { "unset" } else { level }
        );

        let Some(metrics) = xr.instance.exts().meta_performance_metrics else {
            log::info!("b0 performance metrics: absent");

            return Ok(None);
        };

        let state = PerformanceMetricsStateMETA {
            ty: PerformanceMetricsStateMETA::TYPE,
            next: ptr::null(),
            enabled: sys::TRUE,
        };

        // SAFETY: `state` is alive for the call.
        check_xr(unsafe { (metrics.set_performance_metrics_state)(session.as_raw(), &state) })?;

        let mut count = 0;

        // SAFETY: a zero capacity writes only the count.
        check_xr(unsafe {
            (metrics.enumerate_performance_metrics_counter_paths)(
                xr.instance.as_raw(),
                0,
                &mut count,
                ptr::null_mut(),
            )
        })?;

        let mut paths = vec![Path::NULL; usize::try_from(count).expect("counts fit in usize")];

        // SAFETY: `paths` holds `count` entries.
        check_xr(unsafe {
            (metrics.enumerate_performance_metrics_counter_paths)(
                xr.instance.as_raw(),
                count,
                &mut count,
                paths.as_mut_ptr(),
            )
        })?;

        paths.truncate(usize::try_from(count).expect("counts fit in usize"));

        let names = paths
            .into_iter()
            .map(|path| xr.instance.path_to_string(path))
            .collect::<StdResult<Vec<_>, _>>()
            .map_err(Error::Xr)?;

        log::info!("b0 performance counters: {}", names.join(" "));

        let counters = COUNTERS
            .into_iter()
            .map(|(label, path)| {
                Ok(Counter {
                    label,
                    path: xr.instance.string_to_path(path).map_err(Error::Xr)?,
                    unit: None,
                    samples: Vec::with_capacity(MEASURED),
                })
            })
            .collect::<Result<_>>()?;

        Ok(Some(FrameTimes {
            metrics,
            counters,
            frames: 0,
        }))
    }

    /// Samples the counters once a rendered frame ends. A counter logs its
    /// p50 and p99 once its samples are in.
    pub fn sample(&mut self, session: &Session<XrVulkan>) -> Result<()> {
        self.frames += 1;

        if self.frames <= WARM_UP {
            return Ok(());
        }

        for counter in &mut self.counters {
            if counter.samples.len() == MEASURED {
                continue;
            }

            let mut value = PerformanceMetricsCounterMETA {
                ty: PerformanceMetricsCounterMETA::TYPE,
                next: ptr::null(),
                counter_flags: PerformanceMetricsCounterFlagsMETA::EMPTY,
                counter_unit: PerformanceMetricsCounterUnitMETA::GENERIC,
                uint_value: 0,
                float_value: 0.0,
            };

            // SAFETY: `value` is alive for the call.
            check_xr(unsafe {
                (self.metrics.query_performance_metrics_counter)(
                    session.as_raw(),
                    counter.path,
                    &mut value,
                )
            })?;

            // A counter the runtime has not filled yet gives no sample.
            let sample = if value
                .counter_flags
                .contains(PerformanceMetricsCounterFlagsMETA::FLOAT_VALUE_VALID)
            {
                f64::from(value.float_value)
            } else if value
                .counter_flags
                .contains(PerformanceMetricsCounterFlagsMETA::UINT_VALUE_VALID)
            {
                f64::from(value.uint_value)
            } else {
                continue;
            };

            match counter.unit {
                None => counter.unit = Some(value.counter_unit),

                Some(unit) if unit != value.counter_unit => {
                    return Err(Error::CounterUnit(counter.label));
                }

                Some(_) => {}
            }

            counter.samples.push(sample);

            if counter.samples.len() == MEASURED {
                counter.samples.sort_by(f64::total_cmp);

                log::info!(
                    "b0 {} frame time over {MEASURED} frames: p50 {:.3}, p99 {:.3}, {:?}",
                    counter.label,
                    percentile(&counter.samples, 50),
                    percentile(&counter.samples, 99),
                    value.counter_unit
                );
            }
        }

        Ok(())
    }
}

/// The nearest-rank `percent` percentile of the sorted `samples`.
fn percentile(samples: &[f64], percent: usize) -> f64 {
    samples[(samples.len() * percent).div_ceil(100) - 1]
}
