use core::marker::PhantomData;

use ef_driver_common::color::DisplayColor;
use embedded_hal::delay::DelayNs;
use mipidsi::{
    dcs::{
        BitsPerPixel, ExitSleepMode, InterfaceExt, PixelFormat, SetAddressMode, SetDisplayOn,
        SetInvertMode, SetPixelFormat, SoftReset,
    },
    interface::Interface,
    models::Model,
    options::ModelOptions,
};

/// A [`Model`] implementation for a `ST7701S` display.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct St7701sModel<COLOR: DisplayColor>(pub PhantomData<COLOR>);

impl<COLOR: DisplayColor> Model for St7701sModel<COLOR> {
    type ColorFormat = COLOR;

    const FRAMEBUFFER_SIZE: (u16, u16) = (480, 480);

    fn init<DELAY: DelayNs, DI: Interface>(
        &mut self,
        di: &mut DI,
        delay: &mut DELAY,
        options: &ModelOptions,
    ) -> Result<SetAddressMode, DI::Error> {
        // Software reset
        di.write_command(SoftReset)?;
        delay.delay_ms(10);
        // Exit sleep mode
        di.write_command(ExitSleepMode)?;
        delay.delay_ms(200);

        // Set the color inversion mode
        di.write_command(SetInvertMode::new(options.invert_colors))?;

        // TODO: A bunch of initialization commands

        // Set the pixel format
        let format = PixelFormat::with_all(BitsPerPixel::from_rgb_color::<Self::ColorFormat>());
        di.write_command(SetPixelFormat::new(format))?;

        // Set the memory access data control
        let madctl = SetAddressMode::from(options);
        di.write_command(madctl)?;

        // Turn the display on
        di.write_command(SetDisplayOn)?;

        Ok(madctl)
    }
}
