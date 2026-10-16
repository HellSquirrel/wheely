//! Shared pieces for the wheely binaries: sonar ranging, DRV8833 motor control, the
//! obstacle-avoiding cruise state machine and a no-alloc text buffer.
//!
//! Each binary still does its own `bind_interrupts!` and USB logger setup,
//! because those are per-program choices.

#![no_std]

pub mod cruise;
pub mod motor;
pub mod sonar;
pub mod text;
