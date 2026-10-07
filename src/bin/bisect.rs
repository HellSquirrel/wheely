//! Bisect: which part of `drive` stops the motor? The wheel is the readout.
//!
//!   Phase 1 (15 s): motor A alone at 50 %, CONTINUOUS.      Nothing else running.
//!   Phase 2 (15 s): + PWM slice 1 started on GP2/GP3.        Motor A SLOW PULSE, 1 s on / 1 s off.
//!   Phase 3 (15 s): + sonar task pinging on GP16/GP17.       Motor A FAST PULSE, 0.3 s on / 0.3 s off.
//!   Phase 4 (5 s):  coast. Then phases 1 to 3 repeat with everything left running.
//!
//! Spins continuously then dies at the slow pulse  => the GP2/GP3 slice is the problem.
//! Spins through the slow pulse, dies at the fast  => the sonar task is the problem.
//! Never spins                                     => physical; the firmware is not it.
//!
//! Flash: hold BOOTSEL while plugging in, then `cargo run --release --bin bisect`.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::pwm::Pwm;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;
use wheely::motor::{Motor, pwm_config};
use wheely::sonar::{PING_INTERVAL, SonarError, measure};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-bisect"),
    embassy_rp::binary_info::rp_program_description!(c"which part of drive stops the motor"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

static DISTANCE: Signal<CriticalSectionRawMutex, Result<u32, SonarError>> = Signal::new();

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn sonar_task(mut trig: Output<'static>, mut echo: Input<'static>) {
    loop {
        DISTANCE.signal(measure(&mut trig, &mut echo).await);
        Timer::after(PING_INTERVAL).await;
    }
}

/// Run motor A in `pattern` for `secs`, logging once a second.
async fn phase(name: &str, left: &mut Motor<'_>, on_ms: u64, off_ms: u64, secs: u64) {
    log::info!("=== {} for {} s ===", name, secs);
    let end = Instant::now() + Duration::from_secs(secs);
    let mut last_log = Instant::now();
    while Instant::now() < end {
        left.set(500);
        Timer::after_millis(on_ms).await;
        if off_ms > 0 {
            left.coast();
            Timer::after_millis(off_ms).await;
        }
        if last_log.elapsed() >= Duration::from_secs(1) {
            last_log = Instant::now();
            // saturating: after the last sleep `now` can be past `end`, and
            // `end - now` would panic (embassy-time Instant::duration_since unwraps).
            log::info!("{}: {} s left", name, end.saturating_duration_since(Instant::now()).as_secs());
        }
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    let cfg = pwm_config();
    let mut left = Motor::new(Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, cfg.clone()), cfg.clone());

    // Held back until their phase.
    let mut slice1 = Some((p.PWM_SLICE1, p.PIN_2, p.PIN_3));
    let mut sonar_pins = Some((p.PIN_16, p.PIN_17));
    let mut right: Option<Motor<'static>> = None;

    Timer::after_millis(1500).await;
    log::info!("wheely-bisect: watch the wheel. continuous -> slow pulse -> fast pulse -> coast");

    loop {
        phase("phase 1: motor A alone, continuous", &mut left, 1000, 0, 15).await;

        if let Some((s, a, b)) = slice1.take() {
            let mut r = Motor::new(Pwm::new_output_ab(s, a, b, cfg.clone()), cfg.clone());
            r.set(500);
            right = Some(r);
            log::info!("slice 1 on GP2/GP3 started at 50 %");
        }
        phase("phase 2: + slice 1 (GP2/GP3), slow pulse", &mut left, 1000, 1000, 15).await;

        if let Some((t, e)) = sonar_pins.take() {
            spawner.spawn(unwrap!(sonar_task(Output::new(t, Level::Low), Input::new(e, Pull::Down))));
            log::info!("sonar task started");
        }
        phase("phase 3: + sonar, fast pulse", &mut left, 300, 300, 15).await;

        log::info!("=== phase 4: coast 5 s (then repeat with everything running) ===");
        left.coast();
        if let Some(r) = right.as_mut() {
            r.coast();
        }
        Timer::after_secs(5).await;
        if let Some(r) = right.as_mut() {
            r.set(500);
        }
    }
}
