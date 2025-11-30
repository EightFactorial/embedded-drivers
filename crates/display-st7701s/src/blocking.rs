use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use ef_driver_common::{color::DisplayColor, mode::Blocking};
use embedded_hal::delay::DelayNs;

use crate::{CommandDataShifter, St7701s, format_command, format_data};

impl<C: DisplayColor, SPI: WriteOnlyDataCommand, const N: usize> St7701s<C, SPI, Blocking, N> {
    /// Initialize the display.
    ///
    /// # Errors
    ///
    /// Returns an error if communication with the display fails.
    pub fn init<D: DelayNs>(&mut self, _delay: &mut D) -> Result<(), DisplayError> {
        // TODO: Implement initialization sequence
        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

impl<SPI: WriteOnlyDataCommand, const N: usize> WriteOnlyDataCommand
    for CommandDataShifter<SPI, N>
{
    fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        let cmd = match cmd {
            DataFormat::U8(slice) => format_command(slice.iter().copied(), self.1.as_mut_slice()),
            DataFormat::U8Iter(iter) => format_command(iter, self.1.as_mut_slice()),
            _ => return Err(DisplayError::InvalidFormatError),
        };
        self.0.send_commands(DataFormat::U8(cmd))
    }

    fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        let data = match buf {
            DataFormat::U8(slice) => format_data(slice.iter().copied(), self.1.as_mut_slice()),
            DataFormat::U8Iter(iter) => format_data(iter, self.1.as_mut_slice()),
            _ => return Err(DisplayError::InvalidFormatError),
        };
        self.0.send_data(DataFormat::U8(data))
    }
}
