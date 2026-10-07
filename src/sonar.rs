//! RCWL-1601 / HC-SR04 style ultrasonic ranging.
//!
//! Wiring: VCC -> 3V3(OUT), GND -> GND, Trig -> any GPIO output,
//! Echo -> any GPIO input with pull-down. The module is 3.3 V native.

use defmt::Format;
use embassy_rp::gpio::{Input, Output};
use embassy_time::{Duration, Instant, block_for, with_timeout};

/// RCWL-1601 is rated to 450 cm. Past 4 m we call it "nothing there".
pub const MAX_RANGE_MM: u32 = 4000;
/// The module raises Echo well under 1 ms after the trigger. 30 ms of
/// silence means it isn't powered or isn't wired.
pub const ECHO_START_TIMEOUT: Duration = Duration::from_millis(30);
/// 4 m round trip is ~23 ms; the module itself gives up at ~38 ms.
pub const ECHO_END_TIMEOUT: Duration = Duration::from_millis(40);
/// Datasheet asks for >= 60 ms between pings so the last echo has died out.
pub const PING_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub enum SonarError {
    /// Echo never rose: sensor missing, unpowered, or miswired.
    NoEcho,
    /// Echo rose but never fell in time: nothing in range.
    OutOfRange,
    /// Echo was still high from the previous ping when we tried to fire.
    Busy,
}

/// Fire one ping and return the distance in millimetres.
pub async fn measure(trig: &mut Output<'_>, echo: &mut Input<'_>) -> Result<u32, SonarError> {
    // Don't fire on top of an echo that's still in flight.
    if echo.is_high() {
        with_timeout(ECHO_END_TIMEOUT, echo.wait_for_low())
            .await
            .map_err(|_| SonarError::Busy)?;
    }

    // >= 10 µs trigger pulse. Busy-wait on purpose: an async timer wake can
    // land late, and a too-short pulse is the one thing that breaks this.
    trig.set_high();
    block_for(Duration::from_micros(10));
    trig.set_low();

    // Module emits an 8-cycle 40 kHz burst, then raises Echo.
    with_timeout(ECHO_START_TIMEOUT, echo.wait_for_high())
        .await
        .map_err(|_| SonarError::NoEcho)?;
    let start = Instant::now();

    // Echo falls when the reflection comes back.
    with_timeout(ECHO_END_TIMEOUT, echo.wait_for_low())
        .await
        .map_err(|_| SonarError::OutOfRange)?;
    let round_trip_us = start.elapsed().as_micros() as u32;

    // Speed of sound ~343 m/s = 0.343 mm/µs, halved for the round trip.
    // round_trip_us is bounded by ECHO_END_TIMEOUT (~40_000) so this fits u32.
    let mm = round_trip_us * 343 / 2000;
    if mm > MAX_RANGE_MM {
        return Err(SonarError::OutOfRange);
    }
    Ok(mm)
}
