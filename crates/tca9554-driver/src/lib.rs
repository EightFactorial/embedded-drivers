#![doc = include_str!("../README.md")]
#![no_std]

use core::{marker::PhantomData, sync::atomic::AtomicU8};

mod action;
mod device;
mod pin;

/// A driver for a TCA9554 I2C I/O expander.
pub struct TCA9554<I2C, MODE: TCA9554Mode> {
    i2c: I2C,
    address: u8,
    state: AtomicU8,
    _mode: PhantomData<MODE>,
}

impl<I2C, MODE: TCA9554Mode> TCA9554<I2C, MODE> {
    /// Create a new [`TCA9554`] driver.
    #[inline]
    #[must_use]
    pub const fn new(i2c: I2C, address: u8) -> Self {
        Self { i2c, address, state: AtomicU8::new(0), _mode: PhantomData }
    }

    /// Create a new [`TCA9554`] driver assuming an initial state.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the provided `state` value accurately
    /// reflects the actual state of the TCA9554 device.
    pub const unsafe fn new_from_state(i2c: I2C, address: u8, state: u8) -> Self {
        Self { i2c, address, state: AtomicU8::new(state), _mode: PhantomData }
    }

    /// Get the I2C address of the TCA9554 device.
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

/// A marker type for a TCA9554 operating in blocking mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Blocking;
impl TCA9554Mode for Blocking {}

/// A marker type for a TCA9554 operating in asynchronous mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Async;
impl TCA9554Mode for Async {}

/// A marker trait for [`TCA9554`] operating modes.
pub trait TCA9554Mode: sealed::Sealed {}
mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Blocking {}
    impl Sealed for super::Async {}
}
