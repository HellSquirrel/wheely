//! Wheely, step 2: spin motor A forward and back through a DRV8833 with PWM.
//!
//! Bench wiring. Pico stays on USB; the battery pack powers only the driver.
//!   DRV8833 VM       -> battery +
//!   DRV8833 GND      -> battery -  AND a Pico GND pin (common ground)
//!   DRV8833 SLP      -> Pico 3V3(OUT)   physical pin 36
//!   DRV8833 AIN1     -> GP0            physical pin 1   PWM slice 0, channel A
//!   DRV8833 AIN2     -> GP1            physical pin 2   PWM slice 0, channel B
//!   DRV8833 AOUT1/2  -> motor A
//!
//! DRV8833 input-pair truth table:
//!   IN1 = PWM, IN2 = 0    forward at duty
//!   IN1 = 0,   IN2 = PWM  reverse at duty
//!   IN1 = 0,   IN2 = 0    coast
//!   IN1 = 1,   IN2 = 1    brake (not used here)
//!
//! Flash: hold BOOTSEL while plugging in, then `cargo run --release --bin motor`.
//! Logs: `screen /dev/cu.usbmodem<...> 115200`.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::pwm::Pwm;
use wheely::motor::{Motor, Speed, pwm_config};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-motor"),
    embassy_rp::binary_info::rp_program_description!(c"DRV8833 single-motor PWM test"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

/// Linear ramp over 2 s in 20 steps, logging each step.
async fn ramp(motor: &mut Motor<'_>, from: Speed, to: Speed) {
    let step = (to - from) / 20;
    let mut s = from;
    for _ in 0..20 {
        s += step;
        motor.set(s);
        log::info!("speed {:5} permille", s);
        Timer::after_millis(100).await;
    }
    // Integer steps can undershoot; land exactly on the target.
    motor.set(to);
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    // Both compares start at 0, so the motor coasts from the first clock edge.
    let cfg = pwm_config();
    let pwm = Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, cfg.clone());
    let mut motor = Motor::new(pwm, cfg);

    // Give the host a moment to enumerate the USB serial port.
    Timer::after_millis(1500).await;
    let sys_hz = embassy_rp::clocks::clk_sys_freq();
    log::info!(
        "wheely-motor: sys clock {} Hz, pwm top {} => {} Hz carrier",
        sys_hz,
        motor.top(),
        sys_hz / (motor.top() as u32 + 1)
    );

    loop {
        log::info!("forward ramp");
        ramp(&mut motor, 0, 1000).await;
        log::info!("hold full forward");
        Timer::after_secs(1).await;
        log::info!("coast");
        motor.set(0);
        Timer::after_secs(1).await;

        log::info!("reverse ramp");
        ramp(&mut motor, 0, -1000).await;
        log::info!("hold full reverse");
        Timer::after_secs(1).await;
        log::info!("coast");
        motor.set(0);
        Timer::after_secs(2).await;
    }
}
