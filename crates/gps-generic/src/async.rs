use ef_driver_common::mode::Async;
use embedded_io_async::Read;

use crate::{BufferGuard, GenericGps};

impl<UART: Read, const N: usize> GenericGps<UART, Async, N> {
    /// Read a single raw NMEA message from the GPS module.
    ///
    /// # Errors
    ///
    /// Returns an error if the UART read operation fails.
    pub async fn receive_raw(&mut self) -> Result<Option<BufferGuard<'_>>, UART::Error> {
        let buffer = &mut self.buffer[self.index..];
        let received = self.uart.read(buffer).await?;
        self.index += received;

        // Search for a newline, signaling the end of a message.
        for (index, byte) in buffer[..self.index].iter().enumerate() {
            if *byte == b'\n' {
                self.index = 0;
                return Ok(Some(BufferGuard::new(self.buffer.as_mut_slice(), index)));
            }
        }

        Ok(None)
    }

    /// Attempt to read a single NMEA message from the GPS module.
    ///
    /// Returns `None` if a complete message has not yet been received.
    ///
    /// # Errors
    ///
    /// Returns an error if the UART read operation fails or if the message
    /// is malformed.
    pub async fn try_receive_message(&mut self) -> Result<Option<()>, UART::Error> {
        self.receive_raw().await?.map_or(Ok(None), |_buf| Ok(Some(())))
    }

    /// Read a single NMEA message from the GPS module.
    ///
    /// Repeatedly calls [`GenericGps::try_receive_message`] until a complete
    /// message is received.
    ///
    /// # Errors
    ///
    /// Returns an error if the UART read operation fails or if the message
    /// is malformed.
    #[expect(clippy::unit_arg, reason = "WIP")]
    pub async fn receive_message(&mut self) -> Result<(), UART::Error> {
        let mut message = None;
        while message.is_none() {
            message = self.try_receive_message().await?;
        }

        // SAFETY: `message` is guaranteed to be `Some`
        Ok(unsafe { message.unwrap_unchecked() })
    }
}
