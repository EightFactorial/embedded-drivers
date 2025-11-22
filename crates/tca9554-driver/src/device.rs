use embedded_hal::i2c::I2c as BlockingI2c;
use embedded_hal_async::i2c::I2c as AsyncI2c;

use crate::{Async, Blocking, TCA9554};

impl<I2C: BlockingI2c> TCA9554<I2C, Blocking> {
    /// Initialize the [`TCA9554`] device.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails.
    #[inline]
    pub fn init(&mut self) -> Result<(), I2C::Error> {
        crate::action::init_blocking(&mut self.i2c, self.address, &self.state)
    }
}

// -------------------------------------------------------------------------------------------------

impl<I2C: AsyncI2c> TCA9554<I2C, Async> {
    /// Initialize the [`TCA9554`] device.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails.
    #[inline]
    pub fn init(&mut self) -> impl Future<Output = Result<(), I2C::Error>> {
        crate::action::init_async(&mut self.i2c, self.address, &self.state)
    }
}
