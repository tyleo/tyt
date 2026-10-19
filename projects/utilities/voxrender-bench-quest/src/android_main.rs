use crate::{Result, XrApp, XrGpu, XrSystem, log_b1, probe_gpu, probe_indirect, probe_system};
use android_activity::{AndroidApp, MainEvent, PollEvent};
use android_logger::Config;
use log::LevelFilter;
use std::{panic, time::Duration};

/// How long the loop sleeps on Android's events while no session runs.
const IDLE_POLL: Duration = Duration::from_millis(100);

/// The entry point android-activity calls on its main loop thread. The
/// activity finishes when it returns.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("voxrender-bench"),
    );

    // android-activity catches a panic on this thread, and the default hook's
    // message reaches no log.
    panic::set_hook(Box::new(|info| log::error!("{info}")));

    if let Err(error) = run(&app) {
        log::error!("{error}");
    }
}

fn run(app: &AndroidApp) -> Result<()> {
    let android_sdk_version =
        u32::try_from(AndroidApp::sdk_version()).expect("Android API levels are positive");

    let xr = XrSystem::new(app)?;

    probe_system(&xr)?;

    let gpu = XrGpu::new(&xr, android_sdk_version)?;

    probe_gpu(&gpu)?;
    probe_indirect(&gpu)?;
    log_b1(&gpu)?;

    let mut xr_app = XrApp::new(xr, gpu)?;

    let mut destroyed = false;

    while !destroyed {
        let timeout = if xr_app.running() {
            Duration::ZERO
        } else {
            IDLE_POLL
        };

        app.poll_events(Some(timeout), |event| {
            if let PollEvent::Main(MainEvent::Destroy) = event {
                destroyed = true;
            }
        });

        if !xr_app.poll()? {
            break;
        }

        if xr_app.running() {
            xr_app.frame()?;
        }
    }

    Ok(())
}
