#![doc = include_str!("../README.md")]
#![no_std]

use core::marker::PhantomData;

pub use ef_driver_common::color;
use ef_driver_common::{color::DisplayColor, mode::DriverMode};

mod r#async;
mod blocking;
mod command;
#[cfg(feature = "embedded-graphics")]
mod graphics;

/// A driver for a ST7701S display.
pub struct St7701s<C: DisplayColor + ColorFormat, SPI, MODE: DriverMode, const N: usize> {
    spi: CommandDataShifter<SPI, N>,
    _color: PhantomData<C>,
    _mode: PhantomData<MODE>,
}

impl<C: DisplayColor + ColorFormat, SPI, MODE: DriverMode, const N: usize>
    St7701s<C, SPI, MODE, N>
{
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

/// The addressing mode of the display.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AddressMode {
    /// The color order of the display.
    pub color_order: ColorOrder,
    /// Whether the display refreshes forward (false) or backward (true).
    pub refresh_direction: bool,
}

impl AddressMode {
    /// Get the byte-representation of the [`AddressMode`].
    #[must_use]
    pub const fn to_byte(self) -> u8 {
        let mut byte = 0u8;

        match self.color_order {
            ColorOrder::RGB => byte &= 0b1111_1011,
            ColorOrder::BGR => byte |= 0b0000_0100,
        }
        if self.refresh_direction {
            byte &= 0b1111_0111;
        } else {
            byte |= 0b0000_1000;
        }

        byte
    }
}

/// The color order of the display.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ColorOrder {
    /// Red-Green-Blue color order.
    #[default]
    RGB,
    /// Blue-Green-Red color order.
    BGR,
}

/// A trait for color formats supported by the [`St7701s`] driver.
pub trait ColorFormat {
    /// The format byte for the color format.
    const FORMAT_BYTE: u8;
}

impl ColorFormat for color::Rgb565 {
    const FORMAT_BYTE: u8 = 0b0101_0000;
}
impl ColorFormat for color::Rgb666 {
    const FORMAT_BYTE: u8 = 0b0110_0000;
}
impl ColorFormat for color::Rgb888 {
    const FORMAT_BYTE: u8 = 0b0111_0000;
}

// -------------------------------------------------------------------------------------------------

/// A wrapper around an SPI interface that prefixes each byte
/// with either a `0` bit for a command or a `1` bit for data,
/// shifting bits across byte boundaries as needed.
///
/// When writing unaligned data (non-groups of 8 bytes),
/// additional NOP commands (`0x00`) are appended to realign the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDataShifter<SPI, const N: usize>(pub SPI, pub [u8; N]);

/// Format command bytes by properly shifting bits and adding bit prefixes.
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
/// //     v- CMD       v- DATA      v- DATA      v- (CMD + NOP) ...
/// // `[0b00001001, 0b01001101, 0b00101010, 0b11010000, 0b00100000, ...]`
/// let output = format_command(input.into_iter(), &mut buffer);
/// assert_eq!(output, &[0x09, 0x4D, 0x2A, 0xD0, 0x08, 0x04, 0x02, 0x01]);
/// ```
#[doc(hidden)]
pub fn format_command(mut iter: impl Iterator<Item = u8>, buffer: &mut [u8]) -> &[u8] {
    buffer.fill(0);

    let mut bit_carry;
    let mut byte_index = 1usize;

    // The first byte is always prefixed with a `0` bit and shifted
    if let Some(cmd) = iter.next() {
        buffer[0] |= cmd >> 1;
        bit_carry = cmd << 7;
    } else {
        return &buffer[..0];
    }

    // Remaining bytes are prefixed with a `1` bit and shifted
    for byte in iter {
        let shift = byte_index % 8;
        buffer[byte_index] |= bit_carry >> shift | byte >> (shift + 1) | 1u8 << (7 - shift);
        bit_carry = byte << (7 - shift);
        byte_index += 1;
    }

    if !byte_index.is_multiple_of(8) {
        // Append the final byte carry
        buffer[byte_index] = bit_carry | 1u8 << (7 - (byte_index % 8));
        byte_index += 1;

        // Realign to the next byte-group boundary with NOP commands
        while !byte_index.is_multiple_of(8) {
            buffer[byte_index] = 1u8 << (7 - (byte_index % 8));
            byte_index += 1;
        }
    }

    &buffer[..byte_index]
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
/// //     v- DATA      v- DATA      v- DATA      v- (CMD + NOP) ...
/// // `[0b10001001, 0b01001101, 0b00101010, 0b11010000, 0b00100000, ...]`
/// let output = format_data(input.into_iter(), &mut buffer);
/// assert_eq!(output, &[0x89, 0x4D, 0x2A, 0xD0, 0x08, 0x04, 0x02, 0x01]);
/// ```
#[doc(hidden)]
pub fn format_data(mut iter: impl Iterator<Item = u8>, buffer: &mut [u8]) -> &[u8] {
    buffer.fill(0);

    let mut bit_carry;
    let mut byte_index = 1usize;

    // The first byte is always prefixed with a `1` bit and shifted
    if let Some(cmd) = iter.next() {
        buffer[0] |= (cmd >> 1) | 0x80;
        bit_carry = cmd << 7;
    } else {
        return &buffer[..0];
    }

    // Remaining bytes are prefixed with a `1` bit and shifted
    for byte in iter {
        let shift = byte_index % 8;
        buffer[byte_index] |= bit_carry >> shift | byte >> (shift + 1) | 1u8 << (7 - shift);
        bit_carry = byte << (7 - shift);
        byte_index += 1;
    }

    if !byte_index.is_multiple_of(8) {
        // Append the final byte carry
        buffer[byte_index] = bit_carry | 1u8 << (7 - (byte_index % 8));
        byte_index += 1;

        // Realign to the next byte-group boundary with NOP commands
        while !byte_index.is_multiple_of(8) {
            buffer[byte_index] = 1u8 << (7 - (byte_index % 8));
            byte_index += 1;
        }
    }

    &buffer[..byte_index]
}
