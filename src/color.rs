//! Marker types and traits for supported color modes.

pub use embedded_graphics_core::pixelcolor::{Rgb565, Rgb666, Rgb888};
use embedded_graphics_core::prelude::RgbColor;

/// A marker trait for supported color modes.
pub trait DisplayColor: RgbColor + sealed::Sealed {}

impl DisplayColor for Rgb565 {}
impl DisplayColor for Rgb666 {}
impl DisplayColor for Rgb888 {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Rgb565 {}
    impl Sealed for super::Rgb666 {}
    impl Sealed for super::Rgb888 {}
}
