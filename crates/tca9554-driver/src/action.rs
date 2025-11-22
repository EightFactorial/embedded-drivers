use core::sync::atomic::AtomicU8;

use embedded_hal::i2c::I2c as BlockingI2c;
use embedded_hal_async::i2c::I2c as AsyncI2c;

pub(crate) fn init_blocking<I2C: BlockingI2c>(
    _i2c: &mut I2C,
    _address: u8,
    _state: &AtomicU8,
) -> Result<(), I2C::Error> {
    todo!()
}

pub(crate) async fn init_async<I2C: AsyncI2c>(
    _i2c: &mut I2C,
    _address: u8,
    _state: &AtomicU8,
) -> Result<(), I2C::Error> {
    todo!()
}

// -------------------------------------------------------------------------------------------------
