use display_interface::{DisplayError, WriteOnlyDataCommand};
use ef_driver_common::{color::DisplayColor, mode::Blocking};
use embedded_graphics_core::{pixelcolor::raw::ToBytes, prelude::*};

use crate::{ColorFormat, St7701s};

impl<C: DisplayColor + ColorFormat, SPI: WriteOnlyDataCommand, const N: usize> OriginDimensions
    for St7701s<C, SPI, Blocking, N>
{
    fn size(&self) -> Size { Size::new_equal(480) }
}

impl<
    C: DisplayColor + ColorFormat + ToBytes<Bytes = B>,
    B: AsRef<[u8]>,
    SPI: WriteOnlyDataCommand,
    const N: usize,
> DrawTarget for St7701s<C, SPI, Blocking, N>
{
    type Color = C;
    type Error = DisplayError;

    #[expect(unused_variables, reason = "WIP")]
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "Within bounds")]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            // Skip pixels that are out of bounds
            if pixel.0 < Point::zero() || pixel.0 > Point::new_equal(480) {
                continue;
            }

            let (x, y) = (pixel.0.x as u16, pixel.0.y as u16);
            let data: B = <C as ToBytes>::to_be_bytes(pixel.1);
            // self.write_to_address_window(x, y, x, y, data.as_ref())?;
        }
        Ok(())
    }
}
