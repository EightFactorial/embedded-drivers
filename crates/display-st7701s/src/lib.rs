#![doc = include_str!("../README.md")]
#![no_std]

use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use ef_driver_common::color;
use ef_driver_common::color::DisplayColor;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
pub use mipidsi::interface;
use mipidsi::{
    Builder, Display,
    interface::{Interface, InterfacePixelFormat},
};

mod model;
pub use model::St7701sModel;

/// A driver for a ST7701S display.
///
/// This is just a transparent wrapper around a [`Display`] using a
/// [`St7701sModel`] and a [`ParallelInterface`].
#[repr(transparent)]
pub struct St7701s<
    COLOR: DisplayColor + InterfacePixelFormat<DI::Word>,
    DI: Interface,
    RST: OutputPin,
>(Display<DI, St7701sModel<COLOR>, RST>);

impl<COLOR: DisplayColor + InterfacePixelFormat<DI::Word>, DI: Interface, RST: OutputPin>
    St7701s<COLOR, DI, RST>
{
    /// A wrapper around a [`Builder`] to create a [`St7701s`] display driver.
    ///
    /// # Panics
    ///
    /// This function will panic if the initialization sequence fails.
    #[inline]
    #[must_use]
    pub fn new<DELAY: DelayNs>(di: DI, reset: RST, delay: &mut DELAY) -> Self {
        Self(Builder::new(St7701sModel(PhantomData), di).reset_pin(reset).init(delay).unwrap())
    }

    /// Release the inner [`Display`].
    #[inline]
    #[must_use]
    pub fn release(self) -> Display<DI, St7701sModel<COLOR>, RST> { self.0 }
}

impl<COLOR: DisplayColor + InterfacePixelFormat<DI::Word>, DI: Interface, RST: OutputPin> Deref
    for St7701s<COLOR, DI, RST>
{
    type Target = Display<DI, St7701sModel<COLOR>, RST>;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<COLOR: DisplayColor + InterfacePixelFormat<DI::Word>, DI: Interface, RST: OutputPin> DerefMut
    for St7701s<COLOR, DI, RST>
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
