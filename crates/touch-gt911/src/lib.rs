#![doc = include_str!("../README.md")]
#![no_std]

use core::marker::PhantomData;

#[cfg(not(feature = "defmt"))]
use bitflags::bitflags;
#[cfg(feature = "defmt")]
use defmt::bitflags;

mod r#async;
mod blocking;
mod register;

/// A driver for a GT911 touch sensor.
pub struct GT911<I2C, MODE = Touch> {
    i2c: I2C,
    address: u8,
    _mode: PhantomData<MODE>,
}

impl<I2C> GT911<I2C, Touch> {
    /// Create a new [`GT911`] driver in touch mode.
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, address: u8) -> Self { Self { i2c, address, _mode: PhantomData } }
}

impl<I2C, MODE> GT911<I2C, MODE> {
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
    pub const fn i2c_mut(&mut self) -> &mut I2C { &mut self.i2c }

    /// Release the I2C bus.
    #[inline]
    #[must_use]
    pub fn release(self) -> I2C { self.i2c }
}

/// A marker struct for touch mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Touch;
impl GT911Mode for Touch {
    const CLEAR_REGISTER: u16 = register::GT911_STATUS;
}

/// A marker struct for gesture mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Gesture;
impl GT911Mode for Gesture {
    const CLEAR_REGISTER: u16 = register::GT911_GESTURE_STATUS;
}

/// A marker trait for GT911 operating modes.
pub trait GT911Mode: sealed::Sealed {
    /// The register to clear when exiting command mode.
    const CLEAR_REGISTER: u16;
}
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Touch {}
    impl Sealed for super::Gesture {}
}

// -------------------------------------------------------------------------------------------------

/// An error that can occur when using the GT911 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GT911Error<E> {
    /// The device is not ready.
    DeviceNotReady(DetectedTouch),
    /// An invalid touch or gesture point was requested.
    InvalidPoint(u8),
    /// Unexpected product ID.
    ProductIdMismatch([u8; 4], u16),
    /// I2C bus error.
    I2C(E),
}

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
    /// Flags representing the current touch status.
    #[cfg_attr(not(feature = "defmt"), derive(Debug, Clone, Copy, PartialEq, Eq))]
    pub struct DetectedTouch: u8 {
        /// Whether the device is ready.
        const READY_MASK = 0b1000_0000;
        /// Whether a large touch is detected.
        const LARGE_TOUCH_MASK = 0b0100_0000;
        /// Whether the proximity sensor is triggered.
        const PROXIMITY_MASK = 0b0010_0000;
        /// Whether the device is being touched.
        const TOUCH_KEY_MASK = 0b0001_0000;
        /// How many touch points are currently detected.
        const TOUCH_POINT_MASK = 0b0000_1111;
    }
}

impl DetectedTouch {
    /// Returns `true` if the device is ready.
    #[inline]
    #[must_use]
    pub const fn is_ready(self) -> bool { self.contains(DetectedTouch::READY_MASK) }

    /// Returns the number of touch points currently detected.
    #[inline]
    #[must_use]
    pub const fn touch_count(self) -> u8 { self.bits() & DetectedTouch::TOUCH_POINT_MASK.bits() }

    /// Returns `true` if the device has a touch key pressed.
    #[inline]
    #[must_use]
    pub const fn has_touch_key(self) -> bool { self.contains(DetectedTouch::TOUCH_KEY_MASK) }

    /// Returns `true` if a large touch has been detected.
    #[inline]
    #[must_use]
    pub const fn is_large_touched(self) -> bool { self.contains(DetectedTouch::LARGE_TOUCH_MASK) }

    /// Returns `true` if the proximity sensor has been triggered.
    #[inline]
    #[must_use]
    pub const fn is_triggered(self) -> bool { self.contains(DetectedTouch::PROXIMITY_MASK) }
}

/// A gesture detected by the GT911.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DetectedGesture {
    /// No gesture detected.
    None = 0x00,
    /// A character-shaped gesture.
    Char(char) = 0x01,
    /// A swipe to the right.
    SwipeRight = 0xAA,
    /// A swipe to the left.
    SwipeLeft = 0xBB,
    /// A swipe down.
    SwipeDown = 0xAB,
    /// A swipe up.
    SwipeUp = 0xBA,
    /// A double tap.
    DoubleTap = 0xCC,
}

impl DetectedGesture {
    /// Returns `true` if any gesture is detected.
    #[inline]
    #[must_use]
    pub const fn is_any(self) -> bool { !matches!(self, DetectedGesture::None) }

    /// Create a `DetectedGesture` from a raw byte.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            0x27..=0x7F => DetectedGesture::Char(byte as char),
            0xAA => DetectedGesture::SwipeRight,
            0xBB => DetectedGesture::SwipeLeft,
            0xAB => DetectedGesture::SwipeDown,
            0xBA => DetectedGesture::SwipeUp,
            0xCC => DetectedGesture::DoubleTap,
            _ => DetectedGesture::None,
        }
    }
}

/// A gesture point reported by the GT911
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GesturePoint {
    /// The X coordinate
    pub x: u16,
    /// The Y coordinate
    pub y: u16,
}

impl GesturePoint {
    /// Create a gesture point from raw data.
    #[inline]
    #[must_use]
    pub const fn from_bytes(data: [u8; 4]) -> Self {
        Self {
            x: u16::from_le_bytes([data[0], data[1]]),
            y: u16::from_le_bytes([data[2], data[3]]),
        }
    }
}
