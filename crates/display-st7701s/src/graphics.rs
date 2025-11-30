use display_interface::{DisplayError, WriteOnlyDataCommand};
use ef_driver_common::{color::DisplayColor, mode::Blocking};
use embedded_graphics_core::{prelude::*, primitives::Rectangle};

use crate::St7701s;

impl<C: DisplayColor, SPI: WriteOnlyDataCommand, const N: usize> OriginDimensions
    for St7701s<C, SPI, Blocking, N>
{
    fn size(&self) -> Size { Size::new_equal(480) }
}

impl<C: DisplayColor, SPI: WriteOnlyDataCommand, const N: usize> DrawTarget
    for St7701s<C, SPI, Blocking, N>
{
    type Color = C;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, _pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        todo!()
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.draw_iter(area.points().zip(colors).map(|(pos, color)| Pixel(pos, color)))
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_contiguous(area, core::iter::repeat(color))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.fill_solid(&self.bounding_box(), color)
    }
}
