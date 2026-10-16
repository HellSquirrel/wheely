//! Obstacle-avoiding cruise for a robot with one forward-facing sonar.
//!
//! Drive forward. When the sonar gate blocks: back up briefly, spin for a while, stand still
//! long enough for the gate to re-evaluate, then drive on. If the gate blocks again soon after a
//! manoeuvre, spin longer; after two such repeats, spin the other way. The sonar cannot tell
//! left from right, so this is the whole strategy: try, escalate, flip.
//!
//! Pure state machine, no I/O: the caller feeds it the clock and the debounced gate each tick and
//! applies the wheel speeds it returns.

use embassy_time::{Duration, Instant};

use crate::motor::Speed;

/// Reverse for this long before turning, to get some room from whatever we nearly hit.
pub const BACK_TIME: Duration = Duration::from_millis(300);
/// First spin after a block. Doubles on each quick repeat, up to MAX_TURN.
pub const BASE_TURN: Duration = Duration::from_millis(400);
pub const MAX_TURN: Duration = Duration::from_millis(1600);
/// Stand still after a spin so the sonar gate sees a few readings from the new heading.
pub const SETTLE_TIME: Duration = Duration::from_millis(400);
/// A block this soon after the last manoeuvre counts as "still stuck" and escalates.
pub const ESCALATE_WINDOW: Duration = Duration::from_secs(2);
/// Backing and spinning use at least this speed: slow presets won't move the robot on carpet.
pub const MIN_MANOEUVRE_SPEED: Speed = 500;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Forward,
    Backing,
    Turning,
    Settling,
}

impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Phase::Forward => "forward",
            Phase::Backing => "backing",
            Phase::Turning => "turning",
            Phase::Settling => "settling",
        }
    }
}

pub struct Cruise {
    phase: Phase,
    until: Instant,
    /// +1 spins right (left wheel forward), -1 spins left.
    dir: Speed,
    /// Quick repeats since the last clean run; picks the spin length and direction.
    blocks: u8,
    resumed_at: Instant,
}

impl Cruise {
    /// Starts with a settle, so a stale gate can't trigger a manoeuvre on the first tick.
    pub fn new(now: Instant) -> Self {
        Self { phase: Phase::Settling, until: now + SETTLE_TIME, dir: 1, blocks: 0, resumed_at: now }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// One control tick. `blocked` is the debounced sonar gate, `speed` the user's cruising
    /// speed. Returns (left, right) wheel speeds.
    pub fn step(&mut self, now: Instant, blocked: bool, speed: Speed) -> (Speed, Speed) {
        let m = speed.max(MIN_MANOEUVRE_SPEED);
        match self.phase {
            Phase::Forward => {
                if !blocked {
                    return (speed, speed);
                }
                if now.saturating_duration_since(self.resumed_at) < ESCALATE_WINDOW {
                    self.blocks = self.blocks.saturating_add(1);
                } else {
                    self.blocks = 0;
                }
                if self.blocks == 2 {
                    self.dir = -self.dir;
                }
                self.phase = Phase::Backing;
                self.until = now + BACK_TIME;
                (-m, -m)
            }
            Phase::Backing => {
                if now >= self.until {
                    self.phase = Phase::Turning;
                    self.until = now + self.turn_time();
                    self.spin(m)
                } else {
                    (-m, -m)
                }
            }
            Phase::Turning => {
                if now >= self.until {
                    self.phase = Phase::Settling;
                    self.until = now + SETTLE_TIME;
                    (0, 0)
                } else {
                    self.spin(m)
                }
            }
            Phase::Settling => {
                if now >= self.until {
                    self.phase = Phase::Forward;
                    self.resumed_at = now;
                }
                (0, 0)
            }
        }
    }

    /// 400, 800, 1600 ms for 0, 1, 2+ quick repeats.
    fn turn_time(&self) -> Duration {
        let t = BASE_TURN * (1u32 << self.blocks.min(2));
        if t > MAX_TURN { MAX_TURN } else { t }
    }

    fn spin(&self, m: Speed) -> (Speed, Speed) {
        if self.dir > 0 { (m, -m) } else { (-m, m) }
    }
}
