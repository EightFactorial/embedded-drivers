use ef_driver_common::mode::Async;
use embedded_io_async::Read;

use crate::{
    BufferGuard, GenericGps,
    nmea::{NmeaError, NmeaSentence, parse_sentence},
};

impl<UART: Read, const N: usize> GenericGps<UART, Async, N> {
    /// Read a raw message from the GPS module.
    ///
    /// Returns `None` if a complete message has not yet been received.
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

    /// Attempt to read a single NMEA sentence from the GPS module.
    ///
    /// Returns `None` if a complete sentence has not yet been received.
    ///
    /// # Errors
    ///
    /// Returns an error if the UART read operation fails or if the sentence
    /// is malformed.
    pub async fn try_receive_sentence(
        &mut self,
    ) -> Result<Option<NmeaSentence>, NmeaError<UART::Error>> {
        let sentence = self.receive_raw().await.map_err(NmeaError::Other)?;
        sentence.map_or(Ok(None), |buffer| parse_sentence(buffer.as_slice()).map(Some))
    }

    /// Read a single NMEA message from the GPS module.
    ///
    /// Repeatedly calls [`GenericGps::try_receive_sentence`] until a complete
    /// sentence is received.
    ///
    /// # Errors
    ///
    /// Returns an error if the UART read operation fails or if the sentence
    /// is malformed.
    pub async fn receive_sentence(&mut self) -> Result<NmeaSentence, NmeaError<UART::Error>> {
        let mut sentence = None;
        while sentence.is_none() {
            sentence = self.try_receive_sentence().await?;
        }

        // SAFETY: `message` is guaranteed to be `Some`
        Ok(unsafe { sentence.unwrap_unchecked() })
    }
}
