use core::marker::PhantomData;

use embedded_hal_async::i2c::I2c;

use crate::{
    DetectedGesture, DetectedTouch, GT911, GT911Error, GT911Mode, Gesture, GesturePoint, Touch,
    TouchPoint, register,
};

/// A simple macro to enter and exit command mode around a code block.
macro_rules! command_mode {
    ($driver:expr, $mode:ty, $block:block) => {
        command_mode!($driver, $mode, 0, $block)
    };
    ($driver:expr, $mode:ty, $code:expr, $block:block) => {{
        // Enter command mode
        if $code > 7 {
            $driver.write_register_async(register::GT911_COMMAND_CHECK, $code).await?;
        }
        $driver.write_register_async(register::GT911_COMMAND, $code).await?;

        // Create a closure and execute the block (preventing early returns)
        let mut closure = async || $block;
        let result = (closure)().await;

        // Exit command mode
        $driver.write_register_async(<$mode>::CLEAR_REGISTER, 0).await?;

        // Return the result
        result
    }};
}

impl<I2C: I2c, MODE: GT911Mode> GT911<I2C, MODE> {
    /// Query the device's product ID and firmware version.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn device_info_async(&mut self) -> Result<([u8; 4], u16), GT911Error<I2C::Error>> {
        command_mode!(self, MODE, {
            // Query the product ID
            let mut id = [0u8; 4];
            self.read_register_async(register::GT911_PRODUCT_ID1, &mut id).await?;
            // Query the firmware version
            let mut ver = [0u8; 2];
            self.read_register_async(register::GT911_FIRMWARE_VER_LSB, &mut ver).await?;
            Ok((id, u16::from_le_bytes(ver)))
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

impl<I2C: I2c> GT911<I2C, Touch> {
    /// Initialize the GT911 device.
    ///
    /// # Errors
    ///
    /// Returns an error if the device is not ready, if the product ID does not
    /// match, or if any I2C operation fails.
    pub async fn init_async(&mut self) -> Result<(), GT911Error<I2C::Error>> {
        if !self.query_touch_status_async().await?.is_ready() {
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

    /// Reset the device.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    #[expect(clippy::unused_async, reason = "WIP")]
    pub async fn device_reset_async(&mut self) -> Result<(), GT911Error<I2C::Error>> { todo!() }

    /// Query the device's touch status.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn query_touch_status_async(
        &mut self,
    ) -> Result<DetectedTouch, GT911Error<I2C::Error>> {
        command_mode!(self, Touch, {
            // Query the status register
            let mut status = [0u8; 1];
            self.read_register_async(register::GT911_STATUS, &mut status).await?;
            Ok(DetectedTouch::from_bits_truncate(status[0]))
        })
    }

    /// Query the number of active touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    #[inline]
    pub async fn query_touch_count_async(&mut self) -> Result<u8, GT911Error<I2C::Error>> {
        self.query_touch_status_async().await.map(DetectedTouch::touch_count)
    }

    /// Query a specific touch point's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the point index is invalid (>4),
    /// or if any I2C operation fails.
    pub async fn query_touch_async(
        &mut self,
        index: u8,
    ) -> Result<TouchPoint, GT911Error<I2C::Error>> {
        // If the index is higher than the number of points, return an error
        if index > self.query_touch_count_async().await? {
            return Err(GT911Error::InvalidPoint(index));
        }

        let register = match index {
            0 => register::GT911_TOUCH1_TRACK_ID,
            1 => register::GT911_TOUCH2_TRACK_ID,
            2 => register::GT911_TOUCH3_TRACK_ID,
            3 => register::GT911_TOUCH4_TRACK_ID,
            4 => register::GT911_TOUCH5_TRACK_ID,
            // Maximum 5 touch points (0-4)
            _ => unreachable!("Point index out of range"),
        };

        command_mode!(self, Touch, {
            // Query the touch point register
            let mut buf = [0u8; 7];
            self.read_register_async(register, &mut buf).await?;
            Ok(TouchPoint::from_bytes(buf))
        })
    }

    /// Query all active touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn query_touch_all_async(
        &mut self,
    ) -> Result<[Option<TouchPoint>; 5], GT911Error<I2C::Error>> {
        let count = self.query_touch_count_async().await?;
        let mut points = [None, None, None, None, None];
        for i in 0..count {
            points[i as usize] = Some(self.query_touch_async(i).await?);
        }
        Ok(points)
    }

    /// Enter gesture mode.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn gesture_mode_async(
        mut self,
    ) -> Result<GT911<I2C, Gesture>, (Self, GT911Error<I2C::Error>)> {
        if let Err(err) = self.write_register_async(register::GT911_COMMAND_CHECK, 0x8).await {
            return Err((self, err));
        }
        if let Err(err) = self.write_register_async(register::GT911_COMMAND, 0x8).await {
            return Err((self, err));
        }

        let mut gesture: GT911<I2C, Gesture> =
            GT911 { i2c: self.i2c, address: self.address, _mode: PhantomData };

        // Use `init` to verify the mode switch
        match gesture.init_async().await {
            Ok(()) => Ok(gesture),
            Err(err) => {
                Err((GT911 { i2c: gesture.i2c, address: gesture.address, _mode: PhantomData }, err))
            }
        }
    }
}

impl<I2C: I2c> GT911<I2C, Gesture> {
    /// Initialize the GT911 device.
    ///
    /// # Errors
    ///
    /// Returns an error if the device is not ready, if the product ID does not
    /// match, or if any I2C operation fails.
    pub async fn init_async(&mut self) -> Result<(), GT911Error<I2C::Error>> {
        let (id, version) = self.device_info_async().await?;
        if id == [b'G', b'E', b'S', b'T'] {
            Ok(())
        } else {
            // Return that the product ID does not match
            Err(GT911Error::ProductIdMismatch(id, version))
        }
    }

    /// Reset the device, exiting gesture mode.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    #[expect(clippy::unused_async, reason = "WIP")]
    pub async fn device_reset_async(
        self,
    ) -> Result<GT911<I2C, Touch>, (Self, GT911Error<I2C::Error>)> {
        todo!()
    }

    /// Query the detected gesture.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn query_gesture_async(&mut self) -> Result<DetectedGesture, GT911Error<I2C::Error>> {
        command_mode!(self, Gesture, {
            // Query the gesture register
            let mut buf = [0u8; 1];
            self.read_register_async(register::GT911_GESTURE_STATUS, &mut buf).await?;
            Ok(DetectedGesture::from_byte(buf[0]))
        })
    }

    /// Query the number of gesture touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn query_gesture_point_count_async(&mut self) -> Result<u8, GT911Error<I2C::Error>> {
        command_mode!(self, Gesture, {
            // Query the gesture point count register
            let mut buf = [0u8; 1];
            self.read_register_async(register::GT911_GESTURE_TOUCH_POINTS, &mut buf).await?;
            Ok(buf[0])
        })
    }

    /// Query a specific gesture point's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the point index is invalid (>63),
    /// or if any I2C operation fails.
    pub async fn query_gesture_point_async(
        &mut self,
        index: u8,
    ) -> Result<GesturePoint, GT911Error<I2C::Error>> {
        // If the index is higher than the number of points, return an error
        if index > self.query_gesture_point_count_async().await? {
            return Err(GT911Error::InvalidPoint(index));
        }

        let register = match index {
            0..64 => register::GT911_GESTURE_POINT1_X_LSB + u16::from(index) * 4,
            // Maximum 64 touch points (0-63)
            _ => unreachable!("Point index out of range"),
        };

        command_mode!(self, Gesture, {
            // Query the gesture touch point register
            let mut buf = [0u8; 4];
            self.read_register_async(register, &mut buf).await?;
            Ok(GesturePoint::from_bytes(buf))
        })
    }

    /// Query all gesture touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub async fn query_gesture_point_all_async(
        &mut self,
    ) -> Result<[Option<GesturePoint>; 64], GT911Error<I2C::Error>> {
        let count = self.query_gesture_point_count_async().await?;
        let mut points = [None; 64];
        for i in 0..count {
            points[i as usize] = Some(self.query_gesture_point_async(i).await?);
        }
        Ok(points)
    }
}
