use core::marker::PhantomData;

use embedded_hal::i2c::I2c;

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
            $driver.write_register(register::GT911_COMMAND_CHECK, $code)?;
        }
        $driver.write_register(register::GT911_COMMAND, $code)?;

        // Create a closure and execute the block (preventing early returns)
        #[allow(unused_mut, reason = "Closure may need to be mutable")]
        let mut closure = || $block;
        let result = (closure)();

        // Exit command mode
        $driver.write_register(<$mode>::CLEAR_REGISTER, 0)?;

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
    pub fn device_info(&mut self) -> Result<([u8; 4], u16), GT911Error<I2C::Error>> {
        command_mode!(self, MODE, {
            // Query the product ID
            let mut id = [0u8; 4];
            self.read_register(register::GT911_PRODUCT_ID1, &mut id)?;
            // Query the firmware version
            let mut ver = [0u8; 2];
            self.read_register(register::GT911_FIRMWARE_VER_LSB, &mut ver)?;
            Ok((id, u16::from_le_bytes(ver)))
        })
    }

    /// Read from a register.
    ///
    /// # Errors
    ///
    /// Returns an error if the read operation fails.
    fn read_register(
        &mut self,
        register: u16,
        buf: &mut [u8],
    ) -> Result<(), GT911Error<I2C::Error>> {
        self.i2c.write_read(self.address, &register.to_be_bytes(), buf).map_err(GT911Error::I2C)
    }

    /// Write to a register.
    ///
    /// # Errors
    ///
    /// Returns an error if the write operation fails.
    fn write_register(&mut self, register: u16, data: u8) -> Result<(), GT911Error<I2C::Error>> {
        let buf = [register.to_be_bytes()[0], register.to_be_bytes()[1], data];
        self.i2c.write(self.address, &buf).map_err(GT911Error::I2C)
    }
}

impl<I2C: I2c> GT911<I2C, Touch> {
    /// Initialize the GT911 device.
    ///
    /// # Errors
    ///
    /// Returns an error if the device is not ready, if the product ID does not
    /// match, or if any I2C operation fails.
    pub fn init(&mut self) -> Result<(), GT911Error<I2C::Error>> {
        if !self.query_touch_status()?.is_ready() {
            // Return that the device is not ready
            return Err(GT911Error::DeviceNotReady);
        }

        let (id, version) = self.device_info()?;
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
    pub fn device_reset(&mut self) -> Result<(), GT911Error<I2C::Error>> { todo!() }

    /// Query the device's touch status.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub fn query_touch_status(&mut self) -> Result<DetectedTouch, GT911Error<I2C::Error>> {
        command_mode!(self, Touch, {
            // Query the status register
            let mut status = [0u8; 1];
            self.read_register(register::GT911_STATUS, &mut status)?;
            Ok(DetectedTouch::from_bits_truncate(status[0]))
        })
    }

    /// Query the number of active touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    #[inline]
    pub fn query_touch_count(&mut self) -> Result<u8, GT911Error<I2C::Error>> {
        self.query_touch_status().map(DetectedTouch::touch_count)
    }

    /// Query a specific touch point's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the point index is invalid (>4),
    /// or if any I2C operation fails.
    pub fn query_touch(&mut self, index: u8) -> Result<TouchPoint, GT911Error<I2C::Error>> {
        // If the index is higher than the number of points, return an error
        if index > self.query_touch_count()? {
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
            self.read_register(register, &mut buf)?;
            Ok(TouchPoint::from_bytes(buf))
        })
    }

    /// Query all active touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub fn query_touch_all(&mut self) -> Result<[Option<TouchPoint>; 5], GT911Error<I2C::Error>> {
        let count = self.query_touch_count()?;
        let mut points = [None, None, None, None, None];
        for i in 0..count {
            points[i as usize] = Some(self.query_touch(i)?);
        }
        Ok(points)
    }

    /// Enter gesture mode.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    #[expect(clippy::type_complexity, reason = "Returning one of two types of `GT911`")]
    pub fn gesture_mode(mut self) -> Result<GT911<I2C, Gesture>, (Self, GT911Error<I2C::Error>)> {
        let result = self.write_register(register::GT911_COMMAND_CHECK, 0x8);
        let result = result.and_then(|()| self.write_register(register::GT911_COMMAND, 0x8));
        if let Err(err) = result {
            return Err((self, err));
        }

        let mut gesture: GT911<I2C, Gesture> =
            GT911 { i2c: self.i2c, address: self.address, _mode: PhantomData };

        // Use `init` to verify the mode switch
        match gesture.init() {
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
    pub fn init(&mut self) -> Result<(), GT911Error<I2C::Error>> {
        let (id, version) = self.device_info()?;
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
    #[expect(clippy::type_complexity, reason = "Returning one of two types of `GT911`")]
    pub fn device_reset(self) -> Result<GT911<I2C, Touch>, (Self, GT911Error<I2C::Error>)> {
        todo!()
    }

    /// Query the detected gesture.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub fn query_gesture(&mut self) -> Result<DetectedGesture, GT911Error<I2C::Error>> {
        command_mode!(self, Gesture, {
            // Query the gesture register
            let mut buf = [0u8; 1];
            self.read_register(register::GT911_GESTURE_STATUS, &mut buf)?;
            Ok(DetectedGesture::from_byte(buf[0]))
        })
    }

    /// Query the number of gesture touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub fn query_gesture_point_count(&mut self) -> Result<u8, GT911Error<I2C::Error>> {
        command_mode!(self, Gesture, {
            // Query the gesture point count register
            let mut buf = [0u8; 1];
            self.read_register(register::GT911_GESTURE_TOUCH_POINTS, &mut buf)?;
            Ok(buf[0])
        })
    }

    /// Query a specific gesture point's data.
    ///
    /// # Errors
    ///
    /// Returns an error if the point index is invalid (>63),
    /// or if any I2C operation fails.
    pub fn query_gesture_point(
        &mut self,
        index: u8,
    ) -> Result<GesturePoint, GT911Error<I2C::Error>> {
        // If the index is higher than the number of points, return an error
        if index > self.query_gesture_point_count()? {
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
            self.read_register(register, &mut buf)?;
            Ok(GesturePoint::from_bytes(buf))
        })
    }

    /// Query all gesture touch points.
    ///
    /// # Errors
    ///
    /// Returns an error if any I2C operation fails.
    pub fn query_gesture_point_all(
        &mut self,
    ) -> Result<[Option<GesturePoint>; 64], GT911Error<I2C::Error>> {
        let count = self.query_gesture_point_count()?;
        let mut points = [None; 64];
        for i in 0..count {
            points[i as usize] = Some(self.query_gesture_point(i)?);
        }
        Ok(points)
    }
}
