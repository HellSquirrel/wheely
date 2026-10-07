//! Shared pieces for the wheely binaries: sonar ranging and DRV8833 motor control.
//!
//! Each binary still does its own `bind_interrupts!` and USB logger setup,
//! because those are per-program choices.

#![no_std]

pub mod motor;
pub mod sonar;
