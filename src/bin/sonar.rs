//! Wheely, step 1: RCWL-1601 ultrasonic ranging on a Pico 2 W, logged over USB serial.
//!
//! Wiring (RCWL-1601 is 3.3 V native, no level shifting needed):
//!   VCC  -> 3V3(OUT)   Pico physical pin 36
//!   GND  -> GND        any GND pin, e.g. physical pin 38
//!   Trig -> GP16       physical pin 21
//!   Echo -> GP17       physical pin 22
//!
//! Flash without a probe: hold BOOTSEL while plugging the Pico into USB, then
//! `cargo run --release --bin sonar` (also the default `cargo run`). The board reboots
//! into this program and shows up as a USB serial port; `screen /dev/cu.usbmodem<...> 115200`.
//!
//! Note: the Pico 2 W's onboard LED hangs off the CYW43 Wi-Fi chip, not a
//! GPIO, so this program deliberately doesn't blink anything. Logs are the UI.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Timer;
use wheely::sonar::{PING_INTERVAL, measure};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

// Program metadata for `picotool info`. Optional, but cheap.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-sonar"),
    embassy_rp::binary_info::rp_program_description!(c"RCWL-1601 sonar test"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

/// Drains `log::*` records to the USB CDC serial port. 1024-byte buffer;
/// records are dropped, not blocked on, if nobody is reading.
#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    let mut trig = Output::new(p.PIN_16, Level::Low);
    // Pull-down so an unplugged Echo reads low and shows up as NoEcho,
    // rather than a floating pin doing interpretive dance.
    let mut echo = Input::new(p.PIN_17, Pull::Down);

    // Let the sensor's power-up settle, and give the host a moment to
    // enumerate the USB serial port so the first lines aren't lost.
    Timer::after_millis(1500).await;
    log::info!("wheely: sonar test starting");
    info!("wheely: sonar test starting (defmt)");

    loop {
        match measure(&mut trig, &mut echo).await {
            Ok(mm) => log::info!("distance: {} mm ({} cm)", mm, mm / 10),
            Err(e) => log::warn!("sonar: {:?}", e),
        }
        Timer::after(PING_INTERVAL).await;
    }
}
