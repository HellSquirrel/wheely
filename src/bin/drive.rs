//! Wheely, milestone 3: drive forward, coast when the sonar sees something
//! closer than 20 cm, resume when it clears 30 cm.
//!
//! Wiring, in addition to the sonar on GP16/GP17:
//!   DRV8833 AIN1/AIN2 -> GP0/GP1   motor A (left)
//!   DRV8833 BIN1/BIN2 -> GP2/GP3   motor B (right)
//!   DRV8833 SLP -> 3V3, VM -> battery +, GND -> common ground.
//!
//! Safety behaviour: the motors coast whenever the sensor reports a fault
//! (NoEcho, Busy) or stops reporting at all. "Nothing in range" counts as
//! clear road. On the bench this just spins the wheels; on the chassis it drives.
//!
//! Flash: hold BOOTSEL while plugging in, then `cargo run --release --bin drive`.

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
use embassy_time::{Duration, Instant, Timer, with_timeout};
use panic_probe as _;
use wheely::motor::{Motor, Speed, pwm_config};
use wheely::sonar::{PING_INTERVAL, SonarError, measure};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-drive"),
    embassy_rp::binary_info::rp_program_description!(c"drive forward, stop under 20 cm"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/// Coast when the obstacle is closer than this.
const STOP_MM: u32 = 200;
/// Resume only once it is further than this. The gap stops the motors
/// chattering on a reading that hovers around the threshold.
const GO_MM: u32 = 300;
/// Cruise speed, permille. The wheel starts early in the ramp, so this is plenty.
const CRUISE: Speed = 500;
/// If the sonar task goes quiet for this long, coast. Normal cadence is 10 Hz.
const SENSOR_TIMEOUT: Duration = Duration::from_millis(500);
/// Flip to -1 if a motor turns the wrong way once it's on the chassis.
const LEFT_SIGN: Speed = 1;
const RIGHT_SIGN: Speed = 1;

/// Latest sonar result. A Signal keeps only the newest value, which is what
/// a controller wants: stale distances are worse than no distance.
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum State {
    Driving,
    Stopped,
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    // Motors first, so they are known to be coasting before anything else runs.
    let cfg = pwm_config();
    let mut left = Motor::new(Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, cfg.clone()), cfg.clone());
    let mut right = Motor::new(Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_2, p.PIN_3, cfg.clone()), cfg);

    let trig = Output::new(p.PIN_16, Level::Low);
    let echo = Input::new(p.PIN_17, Pull::Down);
    spawner.spawn(unwrap!(sonar_task(trig, echo)));

    Timer::after_millis(1500).await;
    log::info!("wheely-drive: stop < {} mm, go > {} mm, cruise {} permille", STOP_MM, GO_MM, CRUISE);

    let mut state = State::Stopped;
    let mut last_report = Instant::now();
    loop {
        let reading = with_timeout(SENSOR_TIMEOUT, DISTANCE.wait()).await;

        let next = match reading {
            Ok(Ok(mm)) if mm < STOP_MM => State::Stopped,
            Ok(Ok(mm)) if mm > GO_MM => State::Driving,
            Ok(Ok(_)) => state, // in the hysteresis band: keep doing what we were doing
            Ok(Err(SonarError::OutOfRange)) => State::Driving, // nothing ahead
            Ok(Err(e)) => {
                log::warn!("sonar fault {:?}: stopping", e);
                State::Stopped
            }
            Err(_) => {
                log::warn!("sonar silent for {} ms: stopping", SENSOR_TIMEOUT.as_millis());
                State::Stopped
            }
        };

        if next != state {
            state = next;
            match state {
                State::Driving => {
                    log::info!("GO");
                    left.set(CRUISE * LEFT_SIGN);
                    right.set(CRUISE * RIGHT_SIGN);
                }
                State::Stopped => {
                    log::info!("STOP");
                    left.coast();
                    right.coast();
                }
            }
        }

        // Once a second, say what we see, so the log isn't silent while cruising.
        if last_report.elapsed() >= Duration::from_secs(1) {
            last_report = Instant::now();
            match reading {
                Ok(Ok(mm)) => log::info!("{:?}  {} mm", state, mm),
                Ok(Err(e)) => log::info!("{:?}  {:?}", state, e),
                Err(_) => {}
            }
        }
    }
}
