use display_interface::{AsyncWriteOnlyDataCommand, DataFormat, DisplayError};
use ef_driver_common::{color::DisplayColor, mode::Async};
use embedded_hal_async::delay::DelayNs;

use crate::{
    AddressMode, ColorFormat, CommandDataShifter, St7701s, command, format_command, format_data,
};

impl<C: DisplayColor + ColorFormat, SPI: AsyncWriteOnlyDataCommand, const N: usize>
    St7701s<C, SPI, Async, N>
{
    /// Initialize the display.
    ///
    /// # Errors
    ///
    /// Returns an error if communication with the display fails.
    pub async fn init<D: DelayNs>(
        &mut self,
        madctl: AddressMode,
        delay: &mut D,
    ) -> Result<(), DisplayError> {
        // Software reset
        self.spi.send_commands(DataFormat::U8(&[command::ST7701S_SOFT_RESET])).await?;
        delay.delay_ms(150).await; // 150 ms

        // Exit sleep mode
        self.spi.send_commands(DataFormat::U8(&[command::ST7701S_SLEEP_EXIT])).await?;
        delay.delay_ms(150).await; // 150 ms

        // Set the address mode
        self.spi
            .send_commands(DataFormat::U8(&[command::ST7701S_SET_ADDRESS_MODE, madctl.to_byte()]))
            .await?;

        // Turn off color inversion
        self.spi.send_commands(DataFormat::U8(&[command::ST7701S_INVERSION_OFF])).await?;

        // Set the pixel format
        self.spi
            .send_commands(DataFormat::U8(&[command::ST7701S_PIXEL_FORMAT, C::FORMAT_BYTE]))
            .await?;
        delay.delay_ms(10).await; // 10 ms

        // Enter normal mode
        self.spi.send_commands(DataFormat::U8(&[command::ST7701S_NORMAL_MODE])).await?;
        delay.delay_ms(10).await; // 10 ms

        // Exit idle mode
        self.spi.send_commands(DataFormat::U8(&[command::ST7701S_IDLE_OFF])).await?;
        delay.delay_ms(10).await; // 10 ms

        // Turn on the display
        self.spi.send_commands(DataFormat::U8(&[command::ST7701S_DISPLAY_ON])).await?;
        delay.delay_ms(150).await; // 150 ms

        Ok(())
    }
}

// -------------------------------------------------------------------------------------------------

impl<SPI: AsyncWriteOnlyDataCommand, const N: usize> AsyncWriteOnlyDataCommand
    for CommandDataShifter<SPI, N>
{
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        match cmd {
            DataFormat::U8(slice) => {
                // Calculate the number of bytes that can be sent at once.
                // Formatting adds 1 additional byte for every 8 bytes.
                let chunk_size = N * 8 / 9;

                let mut iter = slice.chunks(chunk_size);

                // Initial chunk includes the command byte.
                if let Some(cmd_chunk) = iter.next() {
                    let cmd = format_command(cmd_chunk.iter().copied(), self.1.as_mut_slice());
                    self.0.send_commands(DataFormat::U8(cmd)).await?;
                }

                // Subsequent chunks are data only.
                for chunk in iter {
                    let data = format_data(chunk.iter().copied(), self.1.as_mut_slice());
                    self.0.send_data(DataFormat::U8(data)).await?;
                }

                Ok(())
            }
            DataFormat::U8Iter(iter) => {
                self.0
                    .send_commands(DataFormat::U8(format_command(iter, self.1.as_mut_slice())))
                    .await
            }
            _ => Err(DisplayError::InvalidFormatError),
        }
    }

    async fn send_data(&mut self, dat: DataFormat<'_>) -> Result<(), DisplayError> {
        match dat {
            DataFormat::U8(slice) => {
                // Calculate the number of bytes that can be sent at once.
                // Formatting adds 1 additional byte for every 8 bytes.
                let chunk_size = N * 8 / 9;

                for chunk in slice.chunks(chunk_size) {
                    let data = format_data(chunk.iter().copied(), self.1.as_mut_slice());
                    self.0.send_data(DataFormat::U8(data)).await?;
                }

                Ok(())
            }
            DataFormat::U8Iter(iter) => {
                self.0.send_data(DataFormat::U8(format_data(iter, self.1.as_mut_slice()))).await
            }
            _ => Err(DisplayError::InvalidFormatError),
        }
    }
}
