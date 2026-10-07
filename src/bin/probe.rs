//! Poor man's voltmeter + motor driver exerciser.
//!
//! Runs motor A at 100 % forward on GP0/GP1 (AIN1 solid high, AIN2 low,
//! so a working driver puts VM on AOUT1 and 0 V on AOUT2), and logs the voltage on
//! GP26 / ADC0 (physical pin 31) five times a second.
//!
//! Use: put a jumper in pin 31 and touch its free end to the net you want to
//! read. Expected values with everything wired correctly:
//!   blue rail / any GND          ~0 mV
//!   red rail / SLP pin           ~3300 mV
//!   AIN1 pin row                 ~1650 mV (50 % PWM average)
//!   AIN2 pin row                 ~0 mV
//!   probe lead in the air        random, drifting number = "not connected"
//!
//! NEVER touch VM, AOUT1/AOUT2 or the battery red lead with the probe. Those
//! carry 6 V and the ADC pin is only rated to 3.3 V.
//!
//! Flash: hold BOOTSEL while plugging in, then `cargo run --release --bin probe`.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::adc::{Adc, Channel, Config as AdcConfig, InterruptHandler as AdcInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::Pull;
use embassy_rp::peripherals::USB;
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_rp::usb::{Driver, InterruptHandler as UsbInterruptHandler};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    ADC_IRQ_FIFO => AdcInterruptHandler;
});

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"wheely-probe"),
    embassy_rp::binary_info::rp_program_description!(c"ADC voltmeter on GP26 + motor A at 50%"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

const PWM_HZ: u32 = 20_000;
const DUTY_PERMILLE: u32 = 1000; // 100 %: AIN1 solid high so AOUT1 reads as a DC level

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(unwrap!(logger_task(driver)));

    // Motor A: constant 50 % forward. compare_a = (top + 1) / 2, compare_b = 0.
    let mut cfg = PwmConfig::default();
    let top = (embassy_rp::clocks::clk_sys_freq() / PWM_HZ - 1).min(0xfffe);
    cfg.top = top as u16;
    cfg.compare_a = ((top + 1) * DUTY_PERMILLE / 1000) as u16;
    cfg.compare_b = 0;
    let _pwm = Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, cfg);

    // ADC0 on GP26, no pull so a floating lead is obviously floating.
    let mut adc = Adc::new(p.ADC, Irqs, AdcConfig::default());
    let mut ch = Channel::new_pin(p.PIN_26, Pull::None);

    Timer::after_millis(1500).await;
    log::info!("wheely-probe: motor A at 100% forward on GP0/GP1. Voltmeter on GP26 (pin 31).");
    log::info!("expect: GND ~0, 3V3/SLP/AIN1 ~3300, AIN2 ~0. With VM fed from 3V3: AOUT1 ~3300, AOUT2 ~0.");
    log::info!("do NOT probe VM, AOUT or battery + (6 V).");

    loop {
        // Average 64 samples to tame noise. 12-bit result, 3.3 V reference.
        let mut sum: u32 = 0;
        let mut n: u32 = 0;
        for _ in 0..64 {
            if let Ok(raw) = adc.read(&mut ch).await {
                sum += raw as u32;
                n += 1;
            }
        }
        if n == 0 {
            log::warn!("adc read failed");
        } else {
            let mv = sum * 3300 / 4096 / n;
            log::info!("GP26: {:4} mV", mv);
        }
        Timer::after_millis(200).await;
    }
}
