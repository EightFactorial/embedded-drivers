#![doc = include_str!("../README.md")]
#![no_std]

use core::{marker::PhantomData, ops::Deref};

use ef_driver_common::mode::DriverMode;

mod r#async;
mod blocking;
pub mod nmea;

/// A generic driver for GPS over UART.
pub struct GenericGps<UART, MODE: DriverMode, const N: usize> {
    uart: UART,
    index: usize,
    buffer: [u8; N],
    _mode: PhantomData<MODE>,
}

impl<UART, MODE: DriverMode, const N: usize> GenericGps<UART, MODE, N> {
    /// Create a new [`GenericGps`] driver.
    #[inline]
    #[must_use]
    pub const fn new(uart: UART) -> Self {
        Self { uart, index: 0, buffer: [0u8; N], _mode: PhantomData }
    }

    /// Get a reference to the internal buffer.
    #[inline]
    #[must_use]
    pub const fn buffer(&self) -> &[u8; N] { &self.buffer }

    /// Get a mutable reference to the internal buffer.
    #[inline]
    #[must_use]
    pub const fn buffer_mut(&mut self) -> &mut [u8; N] { &mut self.buffer }

    /// Get a reference to the UART peripheral.
    #[inline]
    #[must_use]
    pub const fn uart(&self) -> &UART { &self.uart }

    /// Get a mutable reference to the UART peripheral.
    #[inline]
    #[must_use]
    pub const fn uart_mut(&mut self) -> &mut UART { &mut self.uart }

    /// Release the UART peripheral.
    #[inline]
    #[must_use]
    pub fn release(self) -> UART { self.uart }
}

// -------------------------------------------------------------------------------------------------

/// A guard that provides access to a portion of the internal buffer.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BufferGuard<'a> {
    buffer: &'a mut [u8],
    end: usize,
}

impl<'a> BufferGuard<'a> {
    /// Create a new [`BufferGuard`].
    #[inline]
    #[must_use]
    pub(crate) const fn new(buffer: &'a mut [u8], end: usize) -> Self { Self { buffer, end } }

    /// Get the range of the buffer that this guard covers.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] { &self.buffer[..self.end] }
}

impl Deref for BufferGuard<'_> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl Drop for BufferGuard<'_> {
    // When the guard is dropped,
    // rotate the buffer to move the unused portion to the front.
    fn drop(&mut self) { self.buffer.rotate_left(self.end); }
}
