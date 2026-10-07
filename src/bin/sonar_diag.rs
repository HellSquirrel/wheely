//! Sonar wiring diagnostic. Tells you WHICH wire is wrong instead of just "NoEcho".
//!
//! Every 2 s it:
//!   1. Reads GP16 and GP17 as inputs with pull-up, then pull-down. A pin that
//!      follows the pull is floating (wire missing, or sensor unpowered). A pin
//!      that stays low under pull-up is being actively driven low, which is what
//!      a powered RCWL-1601's idle Echo output looks like.
//!   2. Fires a ping with GP16 as Trig / GP17 as Echo, then again swapped.
//!      If only the swapped orientation answers, the two wires are swapped.
//!
//! Flash: hold BOOTSEL while plugging in, then `cargo run --release --bin sonar_diag`.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Flex, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::{Duration, Instant, Timer, block_for, with_timeout};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-sonar-diag"),
    embassy_rp::binary_info::rp_program_description!(c"RCWL-1601 wiring diagnostic"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

/// (level with pull-up, level with pull-down)
fn probe_idle(pin: &mut Flex<'_>) -> (bool, bool) {
    pin.set_as_input();
    pin.set_pull(Pull::Up);
    block_for(Duration::from_millis(2));
    let up = pin.is_high();
    pin.set_pull(Pull::Down);
    block_for(Duration::from_millis(2));
    let down = pin.is_high();
    (up, down)
}

fn describe_idle(name: &str, (up, down): (bool, bool)) {
    let verdict = match (up, down) {
        (true, false) => "FLOATING: follows the pull. Wire missing, or sensor unpowered",
        (false, false) => "driven LOW: something is holding it low (a powered sensor's idle Echo does this)",
        (true, true) => "driven HIGH: the RCWL-1601 idles its Trig INPUT high, so this is what a Trig wire looks like",
        (false, true) => "inconsistent: reading flips opposite to the pull; bad contact?",
    };
    log::info!("  {}: pull-up={} pull-down={}  => {}", name, up as u8, down as u8, verdict);
}

/// Returns Ok(round-trip µs) or a short reason.
async fn ping(trig: &mut Flex<'_>, echo: &mut Flex<'_>) -> Result<u32, &'static str> {
    echo.set_as_input();
    echo.set_pull(Pull::Down);
    trig.set_as_output();
    trig.set_low();
    block_for(Duration::from_millis(2));
    if echo.is_high() {
        return Err("echo already high before trigger");
    }
    trig.set_high();
    block_for(Duration::from_micros(10));
    trig.set_low();
    with_timeout(Duration::from_millis(30), echo.wait_for_high())
        .await
        .map_err(|_| "no echo rise within 30 ms")?;
    let t0 = Instant::now();
    with_timeout(Duration::from_millis(40), echo.wait_for_low())
        .await
        .map_err(|_| "echo rose but never fell (out of range?)")?;
    Ok(t0.elapsed().as_micros() as u32)
}

async fn report_ping(label: &str, trig: &mut Flex<'_>, echo: &mut Flex<'_>) -> bool {
    match ping(trig, echo).await {
        Ok(us) => {
            log::info!("  {}: ECHO OK, {} us round trip = {} mm", label, us, us * 343 / 2000);
            true
        }
        Err(why) => {
            log::info!("  {}: {}", label, why);
            false
        }
    }
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    let mut gp16 = Flex::new(p.PIN_16);
    let mut gp17 = Flex::new(p.PIN_17);

    Timer::after_millis(1500).await;
    log::info!("wheely-sonar-diag: expected wiring is Trig->GP16 (pin 21), Echo->GP17 (pin 22)");

    loop {
        log::info!("--- idle levels (both pins as inputs) ---");
        let a = probe_idle(&mut gp16);
        let b = probe_idle(&mut gp17);
        describe_idle("GP16 (should be Trig)", a);
        describe_idle("GP17 (should be Echo)", b);

        log::info!("--- ping, normal orientation: Trig=GP16 Echo=GP17 ---");
        let normal = report_ping("normal", &mut gp16, &mut gp17).await;
        Timer::after_millis(80).await;
        log::info!("--- ping, swapped orientation: Trig=GP17 Echo=GP16 ---");
        let swapped = report_ping("swapped", &mut gp17, &mut gp16).await;

        let verdict = match (normal, swapped, b) {
            (true, _, _) => "WIRING OK. The sonar binary should work.",
            (false, true, _) => "TRIG AND ECHO ARE SWAPPED. Swap the two signal jumpers.",
            (false, false, (true, false)) => "ECHO IS NOT CONNECTED TO A POWERED SENSOR. Check sensor VCC and GND first, then the Echo jumper on pin 22.",
            (false, false, (false, false)) => "SENSOR IS POWERED AND ECHO IS WIRED, but no ping. Check the Trig jumper on pin 21.",
            _ => "Ambiguous. Re-seat all four sensor jumpers.",
        };
        log::info!("VERDICT: {}", verdict);
        log::info!("");
        Timer::after_secs(2).await;
    }
}
