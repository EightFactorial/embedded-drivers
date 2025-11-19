use embedded_hal_async::i2c::I2c;

use crate::{GT911, GT911Error, GT911Status, TouchPoint, register};

/// A simple macro to enter and exit command mode around a block code.
macro_rules! command_mode {
    ($driver:expr, $block:block) => {{
        // Enter command mode
        $driver.write_register_async(register::GT911_COMMAND, 0).await?;

        // Create a closure and execute the block (preventing early returns)
        let mut closure = async || $block;
        let result = (closure)().await;

        // Exit command mode
        $driver.write_register_async(register::GT911_STATUS, 0).await?;

        // Return the result
        result
    }};
}

impl<I2C: I2c> GT911<I2C> {
    /// Initialize the GT911 device.
    ///
    /// # Errors
    ///
    /// Returns an error if the device is not ready, if the product ID does not
    /// match, or if any I2C operation fails.
    pub async fn init_async(&mut self) -> Result<(), GT911Error<I2C::Error>> {
        if !self.device_status_async().await?.is_ready() {
            // Return that the device is not ready
            return Err(GT911Error::DeviceNotReady);
        }

        let (id, version) = self.device_info_async().await?;
        if id == [b'9', b'1', b'1', b'\0'] {
            Ok(())
        } else {
            // Return that the product ID does not match
            Err(GT911Error::ProductIdMismatch(id, version))
        }
    }

    /// Query the device's product ID and firmware version.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn device_info_async(&mut self) -> Result<([u8; 4], u16), GT911Error<I2C::Error>> {
        command_mode!(self, {
            // Query the product ID
            let mut id = [0u8; 4];
            self.read_register_async(register::GT911_PRODUCT_ID1, &mut id).await?;
            // Query the firmware version
            let mut ver = [0u8; 2];
            self.read_register_async(register::GT911_FIRMWARE_VER_LSB, &mut ver).await?;
            Ok((id, u16::from_le_bytes(ver)))
        })
    }

    /// Query the device's status.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn device_status_async(&mut self) -> Result<GT911Status, GT911Error<I2C::Error>> {
        command_mode!(self, {
            // Query the status register
            let mut status = [0u8; 1];
            self.read_register_async(register::GT911_STATUS, &mut status).await?;
            Ok(GT911Status::from_bits_truncate(status[0]))
        })
    }

    /// Query the number of active touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    #[inline]
    pub async fn query_touch_count_async(&mut self) -> Result<u8, GT911Error<I2C::Error>> {
        self.device_status_async().await.map(GT911Status::touch_count)
    }

    /// Query a specific touch point's data.
    ////
    /// # Errors
    ////
    /// Returns an error if the point index is invalid (>4),
    /// or if any I2C operation fails.
    pub async fn query_touch_async(
        &mut self,
        point: u8,
    ) -> Result<TouchPoint, GT911Error<I2C::Error>> {
        let register = match point {
            0 => register::GT911_TOUCH1_TRACK_ID,
            1 => register::GT911_TOUCH2_TRACK_ID,
            2 => register::GT911_TOUCH3_TRACK_ID,
            3 => register::GT911_TOUCH4_TRACK_ID,
            4 => register::GT911_TOUCH5_TRACK_ID,
            _ => return Err(GT911Error::InvalidPoint(point)),
        };

        command_mode!(self, {
            // Query the touch point register
            let mut buf = [0u8; 7];
            self.read_register_async(register, &mut buf).await?;
            Ok(TouchPoint::from_bytes(buf))
        })
    }

    /// Read from a register asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the read operation fails.
    async fn read_register_async(
        &mut self,
        register: u16,
        buf: &mut [u8],
    ) -> Result<(), GT911Error<I2C::Error>> {
        self.i2c
            .write_read(self.address, &register.to_le_bytes(), buf)
            .await
            .map_err(GT911Error::I2C)
    }

    /// Write to a register asynchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the write operation fails.
    async fn write_register_async(
        &mut self,
        register: u16,
        data: u8,
    ) -> Result<(), GT911Error<I2C::Error>> {
        let buf = [register.to_le_bytes()[0], register.to_le_bytes()[1], data];
        self.i2c.write(self.address, &buf).await.map_err(GT911Error::I2C)
    }
}
