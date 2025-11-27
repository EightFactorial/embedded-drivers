#![doc = include_str!("../README.md")]
#![expect(unused_imports, reason = "WIP")]
#![no_std]

#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;
#[cfg(feature = "defmt")]
use defmt::bitflags;
pub use ef_driver_common::mode;
use ef_driver_common::mode::DriverMode;
