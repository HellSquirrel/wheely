//! One brushed DC motor on a DRV8833 input pair that shares a PWM slice.
//!
//! DRV8833 H-bridge logic (per input pair):
//!   IN1 IN2   OUT1 OUT2
//!    0   0     Z    Z    coast (fast decay)
//!    0   1     L    H    reverse
//!    1   0     H    L    forward
//!    1   1     L    L    brake (slow decay)
//!
//! Speed control uses SLOW DECAY: one input is held high and the other is
//! PWMed, so between drive pulses the motor is braked rather than left to
//! coast. Fast decay (PWM one input, other low) gave so little torque at
//! 20 kHz that a geared TT motor would not start below ~100 % duty on the
//! bench (2026-10-06). With slow decay the duty maps far more linearly to
//! speed and low-duty torque is usable.
//!
//!   forward at duty d:  IN1 = 1,       IN2 = PWM low for fraction d
//!   reverse at duty d:  IN1 = PWM low for fraction d,  IN2 = 1
//!   speed 0:            IN1 = 0, IN2 = 0  (coast, so a stopped robot rolls free)
//!
//! Wheely: motor A on GP0/GP1 (slice 0), motor B on GP2/GP3 (slice 1).
//! SLP must be tied high or the chip stays asleep.

use embassy_rp::pwm::{Config as PwmConfig, Pwm};

/// PWM carrier. 20 kHz is above what most people can hear and well under the
/// DRV8833's 250 kHz input limit.
pub const PWM_HZ: u32 = 20_000;

/// Motor speed in permille: -1000 full reverse ..= 1000 full forward, 0 coast.
pub type Speed = i16;

pub struct Motor<'d> {
    pwm: Pwm<'d>,
    cfg: PwmConfig,
}

impl<'d> Motor<'d> {
    /// `pwm` must have been created with `Pwm::new_output_ab` and this `cfg`
    /// (normally from [`pwm_config`]).
    pub fn new(pwm: Pwm<'d>, cfg: PwmConfig) -> Self {
        Self { pwm, cfg }
    }

    pub fn set(&mut self, speed: Speed) {
        let speed = speed.clamp(-1000, 1000);
        // compare == top + 1 is 100 % high (embassy-rp pwm docs), so scale to
        // top + 1. `pwm_config` caps top so top + 1 still fits in u16.
        let full = self.cfg.top as u32 + 1;
        // Drive fraction of each period. In slow decay the PWMed input must be
        // LOW while driving and HIGH while braking, so its compare value is the
        // complement: high for (1 - d) of the period.
        let drive = full * speed.unsigned_abs() as u32 / 1000;
        let brake = (full - drive) as u16; // compare for the PWMed input
        let high = full as u16; // compare == top + 1 is always-high
        let (a, b) = match speed {
            s if s > 0 => (high, brake),
            s if s < 0 => (brake, high),
            _ => (0, 0),
        };
        self.cfg.compare_a = a;
        self.cfg.compare_b = b;
        self.pwm.set_config(&self.cfg);
    }

    pub fn coast(&mut self) {
        self.set(0);
    }

    pub fn top(&self) -> u16 {
        self.cfg.top
    }
}

/// Slice config for [`PWM_HZ`] with the counter running at the system clock,
/// both compares at 0 (coast).
pub fn pwm_config() -> PwmConfig {
    // Default: divider 1, not phase-correct, enabled, both compares 0.
    let mut cfg = PwmConfig::default();
    let sys_hz = embassy_rp::clocks::clk_sys_freq();
    // Period is (top + 1) ticks at sys_hz.
    let top = sys_hz / PWM_HZ - 1;
    // u16 field, and keep top + 1 <= u16::MAX for the duty math in `Motor::set`.
    cfg.top = top.min(0xfffe) as u16;
    cfg
}
