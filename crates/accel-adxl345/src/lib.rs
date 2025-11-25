#![doc = include_str!("../README.md")]
#![no_std]

use core::marker::PhantomData;

#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;
#[cfg(feature = "defmt")]
use defmt::bitflags;
use ef_driver_common::mode::DriverMode;

mod r#async;
mod blocking;
mod register;

/// A driver for a ADXL345 accelerometer.
pub struct Adxl345<I2C, MODE: DriverMode> {
    i2c: I2C,
    address: u8,
    _mode: PhantomData<MODE>,
}

impl<I2C, MODE: DriverMode> Adxl345<I2C, MODE> {
    /// Create a new [`Adxl345`] driver instance.
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, address: u8) -> Self { Self { i2c, address, _mode: PhantomData } }

    /// Get the I2C address of the [`Adxl345`] device.
    #[inline]
    #[must_use]
    pub const fn address(&self) -> u8 { self.address }

    /// Get a reference to the I2C bus.
    #[inline]
    #[must_use]
    pub const fn i2c(&self) -> &I2C { &self.i2c }

    /// Get a mutable reference to the I2C bus.
    #[inline]
    #[must_use]
    pub const fn i2c_mut(&mut self) -> &mut I2C { &mut self.i2c }

    /// Release the I2C bus.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}

// -------------------------------------------------------------------------------------------------

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[expect(missing_docs, reason = "Self explanatory")]
pub enum GRange {
    #[default]
    Two = 0b00,
    Four = 0b01,
    Eight = 0b10,
    Sixteen = 0b11,
}

impl GRange {
    /// Create a [`GRange`] from a byte value.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        match byte & DataFormat::RANGE_MASK.bits() {
            0b00 => GRange::Two,
            0b01 => GRange::Four,
            0b10 => GRange::Eight,
            0b11 => GRange::Sixteen,
            _ => unreachable!(),
        }
    }
}

bitflags! {
    #[cfg_attr(not(feature = "defmt"), derive(Debug, Clone, Copy, PartialEq, Eq))]
    struct DataFormat: u8 {
        const SELF_TEST = 0b1000_0000;
        const SPI_MODE = 0b0100_0000;
        const INTERRUPT_INVERT = 0b0010_0000;
        const FULL_RESOLUTION = 0b0000_1000;
        const JUSTIFY = 0b0000_0100;
        const RANGE_MASK = 0b0000_0011;
    }
}

#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[expect(missing_docs, reason = "Self explanatory")]
pub enum DataRate {
    Hz0_10 = 0b0000,
    Hz0_20 = 0b0001,
    Hz0_39 = 0b0010,
    Hz0_78 = 0b0011,
    Hz1_56 = 0b0100,
    Hz3_13 = 0b0101,
    Hz6_25 = 0b0110,
    Hz12_5 = 0b0111,
    Hz25 = 0b1000,
    Hz50 = 0b1001,
    #[default]
    Hz100 = 0b1010,
    Hz200 = 0b1011,
    Hz400 = 0b1100,
    Hz800 = 0b1101,
    Hz1600 = 0b1110,
    Hz3200 = 0b1111,
}

bitflags! {
    #[cfg_attr(not(feature = "defmt"), derive(Debug, Clone, Copy, PartialEq, Eq))]
    struct FifoControl: u8 {
        const FIFO_MASK = 0b1100_0000;
        const TRIGGER = 0b0010_0000;
        const SAMPLES_MASK = 0b0001_1111;
    }
}

bitflags! {
    #[cfg_attr(not(feature = "defmt"), derive(Debug, Clone, Copy, PartialEq, Eq))]
    struct FifoStatus: u8 {
        const TRIGGER = 0b1000_0000;
        const ENTRY_MASK = 0b0011_1111;
    }
}

/// FIFO operation modes
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FifoMode {
    /// Bypass FIFO
    #[default]
    Bypass = 0b00,
    /// Collect data in FIFO mode,
    /// only collecting new data when FIFO is not full
    Fifo = 0b01,
    /// Collect data in FIFO mode,
    /// overwriting the oldest data when FIFO is full
    Stream = 0b10,
    /// Collect data in FIFO mode when triggered,
    /// only collecting new data when FIFO is not full
    Trigger = 0b11,
}

impl FifoMode {
    /// Create a [`FifoMode`] from a byte value.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        match byte & FifoControl::FIFO_MASK.bits() {
            0b00 => FifoMode::Bypass,
            0b01 => FifoMode::Fifo,
            0b10 => FifoMode::Stream,
            0b11 => FifoMode::Trigger,
            _ => unreachable!(),
        }
    }
}
