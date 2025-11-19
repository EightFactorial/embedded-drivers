#![doc = include_str!("../README.md")]
#![no_std]

use bitflags::bitflags;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "async")]
mod r#async;
mod blocking;
mod register;

/// A driver for a GT911 touch sensor.
pub struct GT911<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> GT911<I2C> {
    /// Create a new [`GT911`] driver.
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, address: u8) -> Self { GT911 { i2c, address } }

    /// Get the I2C address of the GT911 device.
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
    pub fn i2c_mut(&mut self) -> &mut I2C { &mut self.i2c }

    /// Release the I2C bus.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}

/// An error that can occur when using the GT911 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GT911Error<E> {
    /// The device is not ready.
    DeviceNotReady,
    /// An invalid touch point was requested.
    InvalidPoint(u8),
    /// Unexpected product ID.
    ProductIdMismatch([u8; 4], u16),
    /// I2C bus error.
    I2C(E),
}

// -------------------------------------------------------------------------------------------------

/// A touch point reported by the GT911.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchPoint {
    /// The touch point ID
    pub point: u8,
    /// The X coordinate
    pub x: u16,
    /// The Y coordinate
    pub y: u16,
    /// The area of the touch
    pub area: u16,
}

impl TouchPoint {
    /// Create a touch point from raw data.
    #[must_use]
    pub const fn from_bytes(data: [u8; 7]) -> Self {
        Self {
            point: data[0],
            x: u16::from_le_bytes([data[1], data[2]]),
            y: u16::from_le_bytes([data[3], data[4]]),
            area: u16::from_le_bytes([data[5], data[6]]),
        }
    }
}

bitflags! {
    /// The status of the GT911 device.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct GT911Status: u8 {
        /// Whether the device is ready.
        const READY_MASK = 0b1000_0000;
        /// Whether a large touch is detected.
        const LARGE_TOUCH_MASK = 0b0100_0000;
        /// Whether the proximity sensor is triggered.
        const PROXIMITY_MASK = 0b0010_0000;
        /// Whether the device is being touched.
        const TOUCH_MASK = 0b0001_0000;
        /// How many touch points are currently detected.
        const TOUCH_POINT_MASK = 0b0000_1111;
    }

}

impl GT911Status {
    /// Returns `true` if the device is ready.
    #[inline]
    #[must_use]
    pub const fn is_ready(self) -> bool { self.contains(GT911Status::READY_MASK) }

    /// Returns the number of touch points currently detected.
    #[inline]
    #[must_use]
    pub const fn touch_count(self) -> u8 { self.bits() & GT911Status::TOUCH_POINT_MASK.bits() }

    /// Returns `true` if the device is being touched.
    #[inline]
    #[must_use]
    pub const fn is_touched(self) -> bool { self.contains(GT911Status::TOUCH_MASK) }

    /// Returns `true` if a large touch has been detected.
    #[inline]
    #[must_use]
    pub const fn is_large_touched(self) -> bool { self.contains(GT911Status::LARGE_TOUCH_MASK) }

    /// Returns `true` if the proximity sensor has been triggered.
    #[inline]
    #[must_use]
    pub const fn is_triggered(self) -> bool { self.contains(GT911Status::PROXIMITY_MASK) }
}
