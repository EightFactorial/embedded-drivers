#![doc = include_str!("../README.md")]
#![no_std]

extern crate std;

use core::marker::PhantomData;

use ef_driver_common::{color::DisplayColor, mode::DriverMode};

mod r#async;
mod blocking;
mod command;
#[cfg(feature = "embedded-graphics")]
mod graphics;

/// A driver for a ST7701S display.
pub struct St7701s<C: DisplayColor, SPI, MODE: DriverMode, const N: usize> {
    spi: CommandDataShifter<SPI, N>,
    _color: PhantomData<C>,
    _mode: PhantomData<MODE>,
}

impl<C: DisplayColor, SPI, MODE: DriverMode, const N: usize> St7701s<C, SPI, MODE, N> {
    /// Create a new [`St7701s`] driver instance.
    #[inline]
    #[must_use]
    pub const fn new(spi: SPI) -> Self { Self::new_with_buffer(spi, [0; N]) }

    /// Create a new [`St7701s`] driver instance.
    #[inline]
    #[must_use]
    pub const fn new_with_buffer(spi: SPI, buffer: [u8; N]) -> Self {
        Self { spi: CommandDataShifter(spi, buffer), _color: PhantomData, _mode: PhantomData }
    }

    /// Get a reference to the SPI interface.
    #[inline]
    #[must_use]
    pub const fn spi(&self) -> &SPI { &self.spi.0 }

    /// Get a mutable reference to the SPI interface.
    #[inline]
    #[must_use]
    pub const fn spi_mut(&mut self) -> &mut SPI { &mut self.spi.0 }

    /// Release the SPI interface.
    #[inline]
    #[must_use]
    pub fn release(self) -> SPI { self.spi.0 }
}

// -------------------------------------------------------------------------------------------------

/// A wrapper around an SPI interface that prefixes each byte
/// with either a `0` bit for commands or a `1` bit for data,
/// shifting bits across byte boundaries as needed.
///
/// When writing unaligned data (non-groups of 8 bytes),
/// additional NOPs (`0x00`) are appended to realign the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDataShifter<SPI, const N: usize>(pub SPI, pub [u8; N]);

/// Format command bytes by properly shifting bits and adding byte prefixes.
///
/// # Example
///
/// ```rust
/// use ef_st7701s::format_command;
///
/// let mut buffer = [0u8; 10];
///
/// // `[0b00010010, 0b00110100, 0b01010110]`
/// let input = [0x12, 0x34, 0x56];
/// // `[0b00001001, 0b10001101, 0b10001010, 0b10001100, 0b00000000, 0b00000000, 0b00000000, 0b00000000]`
/// let output = format_command(input.into_iter(), &mut buffer);
/// assert_eq!(output, &[0x09, 0x8D, 0x8A, 0x8C, 0x00, 0x00, 0x00, 0x00]);
/// ```
#[doc(hidden)]
pub fn format_command(mut iter: impl Iterator<Item = u8>, buffer: &mut [u8]) -> &[u8] {
    buffer.fill(0);

    let mut bit_carry;
    let mut byte_index = 1usize;

    // The first byte is always prefixed with a `0` bit
    if let Some(cmd) = iter.next() {
        buffer[0] |= cmd >> 1;
        bit_carry = cmd << 7;
    } else {
        return &buffer[..0];
    }

    // All remaining bytes are prefixed with a `1` bit and shifted across boundaries
    for byte in iter {
        let shift = (byte_index + 1) % 8;
        buffer[byte_index] |= bit_carry | (byte >> shift) | 0b1000_0000;
        bit_carry = byte << (8 - shift);
        byte_index += 1;
    }

    // Append any remaining bits
    buffer[byte_index] = bit_carry >> ((byte_index + 1) % 8) | 0b1000_0000;
    byte_index += 1;

    // Return an aligned slice
    &buffer[..byte_index.next_multiple_of(8)]
}

/// Format data bytes by properly shifting bits and adding byte prefixes.
///
/// # Example
///
/// ```rust
/// use ef_st7701s::format_data;
///
/// let mut buffer = [0u8; 10];
///
/// // `[0b00010010, 0b00110100, 0b01010110]`
/// let input = [0x12, 0x34, 0x56];
/// // `[0b10001001, 0b10001101, 0b10001010, 0b10001100, 0b00000000, 0b00000000, 0b00000000, 0b00000000]`
/// let output = format_data(input.into_iter(), &mut buffer);
/// assert_eq!(output, &[0x89, 0x8D, 0x8A, 0x8C, 0x00, 0x00, 0x00, 0x00]);
/// ```
#[doc(hidden)]
pub fn format_data(mut iter: impl Iterator<Item = u8>, buffer: &mut [u8]) -> &[u8] {
    buffer.fill(0);

    let mut bit_carry;
    let mut byte_index = 1usize;

    // The first byte is always prefixed with a `1` bit
    if let Some(cmd) = iter.next() {
        buffer[0] |= (cmd >> 1) | 0b1000_0000;
        bit_carry = cmd << 7;
    } else {
        return &buffer[..0];
    }

    // All remaining bytes are prefixed with a `1` bit and shifted across boundaries
    for byte in iter {
        let shift = (byte_index + 1) % 8;
        buffer[byte_index] |= bit_carry | (byte >> shift) | 0b1000_0000;
        bit_carry = byte << (8 - shift);
        byte_index += 1;
    }

    // Append any remaining bits
    buffer[byte_index] = bit_carry >> ((byte_index + 1) % 8) | 0b1000_0000;
    byte_index += 1;

    // Return an aligned slice
    &buffer[..byte_index.next_multiple_of(8)]
}
