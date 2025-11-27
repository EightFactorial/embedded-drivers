#![doc = include_str!("../README.md")]
#![no_std]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use ef_driver_common::color;
use ef_driver_common::color::DisplayColor;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use mipidsi::{
    Builder, Display,
    interface::{InterfacePixelFormat, OutputBus, ParallelInterface},
};

mod model;
pub use model::St7701sModel;

/// A driver for a ST7701S display.
///
/// This is just a transparent wrapper around a [`Display`] using a
/// [`St7701sModel`] and a [`ParallelInterface`].
#[repr(transparent)]
pub struct St7701s<
    COLOR: DisplayColor + InterfacePixelFormat<BUS::Word>,
    BUS: OutputBus,
    DC: OutputPin,
    WR: OutputPin,
    RST: OutputPin,
>(Display<ParallelInterface<BUS, DC, WR>, St7701sModel<COLOR>, RST>)
where
    BUS::Word: Eq + From<u8>;

impl<
    COLOR: DisplayColor + InterfacePixelFormat<BUS::Word>,
    BUS: OutputBus,
    DC: OutputPin,
    WR: OutputPin,
    RST: OutputPin,
> St7701s<COLOR, BUS, DC, WR, RST>
where
    BUS::Word: Eq + From<u8>,
{
    /// A wrapper around a [`Builder`] to create a [`St7701s`] display driver.
    ///
    /// # Panics
    ///
    /// This function will panic if the initialization sequence fails.
    #[inline]
    #[must_use]
    pub fn new<DELAY: DelayNs>(bus: BUS, dc: DC, wr: WR, reset: RST, delay: &mut DELAY) -> Self {
        Self(
            Builder::new(St7701sModel(PhantomData), ParallelInterface::new(bus, dc, wr))
                .reset_pin(reset)
                .init(delay)
                .unwrap(),
        )
    }

    /// Release the inner [`Display`].
    #[inline]
    #[must_use]
    pub fn release(self) -> Display<ParallelInterface<BUS, DC, WR>, St7701sModel<COLOR>, RST> {
        self.0
    }
}

impl<
    COLOR: DisplayColor + InterfacePixelFormat<BUS::Word>,
    BUS: OutputBus,
    DC: OutputPin,
    WR: OutputPin,
    RST: OutputPin,
> Deref for St7701s<COLOR, BUS, DC, WR, RST>
where
    BUS::Word: Eq + From<u8>,
{
    type Target = Display<ParallelInterface<BUS, DC, WR>, St7701sModel<COLOR>, RST>;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<
    COLOR: DisplayColor + InterfacePixelFormat<BUS::Word>,
    BUS: OutputBus,
    DC: OutputPin,
    WR: OutputPin,
    RST: OutputPin,
> DerefMut for St7701s<COLOR, BUS, DC, WR, RST>
where
    BUS::Word: Eq + From<u8>,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
