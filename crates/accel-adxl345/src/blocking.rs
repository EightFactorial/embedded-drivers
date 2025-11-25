use ef_driver_common::mode::Blocking;
use embedded_hal::i2c::I2c;

use crate::{
    Adxl345, BWRate, DataFormat, DataRate, FifoControl, FifoMode, FifoStatus, GRange, PowerControl,
    register,
};

impl<I2C: I2c> Adxl345<I2C, Blocking> {
    /// Read the device ID
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_device_id(&mut self) -> Result<u8, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_DEVICE_ID, &mut buf)?;
        Ok(buf[0])
    }

    /// Get the acceleration data for X, Y, and Z axes
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_acceleration(&mut self) -> Result<(i16, i16, i16), I2C::Error> {
        let mut buf = [0u8; 6];
        self.read_register(register::ADXL345_DATA_X_LSB, &mut buf)?;
        let x = i16::from_le_bytes([buf[0], buf[1]]);
        let y = i16::from_le_bytes([buf[2], buf[3]]);
        let z = i16::from_le_bytes([buf[4], buf[5]]);
        Ok((x, y, z))
    }

    /// Get the offset values for X, Y, and Z axes
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    #[expect(clippy::cast_possible_wrap, reason = "This is desired behavior")]
    pub fn get_axis_offsets(&mut self) -> Result<(i8, i8, i8), I2C::Error> {
        let mut buf = [0u8; 3];
        self.read_register(register::ADXL345_OFFSET_X, &mut buf)?;
        Ok((buf[0] as i8, buf[1] as i8, buf[2] as i8))
    }

    /// Set the offset values for X, Y, and Z axes
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    #[expect(clippy::cast_sign_loss, reason = "This is desired behavior")]
    pub fn set_axis_offsets(&mut self, x: i8, y: i8, z: i8) -> Result<(), I2C::Error> {
        self.write_register(register::ADXL345_OFFSET_X, x as u8)?;
        self.write_register(register::ADXL345_OFFSET_Y, y as u8)?;
        self.write_register(register::ADXL345_OFFSET_Z, z as u8)?;
        Ok(())
    }

    /// Get the device's low power mode state.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_low_power_mode(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_BW_RATE, &mut buf)?;
        let bwrate = BWRate::from_bits_truncate(buf[0]);
        Ok(bwrate.contains(BWRate::LOW_POWER))
    }

    /// Set the device's low power mode state.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_low_power_mode(&mut self, low_power: bool) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_BW_RATE, &mut buf)?;
        let mut bwrate = BWRate::from_bits_truncate(buf[0]);
        bwrate.set(BWRate::LOW_POWER, low_power);
        self.write_register(register::ADXL345_BW_RATE, bwrate.bits())
    }

    /// Get the device's data rate.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_data_rate(&mut self) -> Result<DataRate, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_BW_RATE, &mut buf)?;
        Ok(DataRate::from_byte(buf[0]))
    }

    /// Set the device's data rate.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_data_rate(&mut self, rate: DataRate) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_BW_RATE, &mut buf)?;
        let mut bwrate = BWRate::from_bits_truncate(buf[0]);
        bwrate.remove(BWRate::RATE_MASK);
        bwrate.insert(BWRate::from_bits_truncate(rate as u8));
        self.write_register(register::ADXL345_BW_RATE, bwrate.bits())
    }

    /// Get whether the device is in link mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_link_mode(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_POWER_CONTROL, &mut buf)?;
        let power_ctrl = PowerControl::from_bits_truncate(buf[0]);
        Ok(power_ctrl.contains(PowerControl::LINK))
    }

    /// Set whether the device is in link mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_link_mode(&mut self, link: bool) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_POWER_CONTROL, &mut buf)?;
        let mut power_ctrl = PowerControl::from_bits_truncate(buf[0]);
        power_ctrl.set(PowerControl::LINK, link);
        self.write_register(register::ADXL345_POWER_CONTROL, power_ctrl.bits())
    }

    /// Get whether the device has auto sleep enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_auto_sleep(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_POWER_CONTROL, &mut buf)?;
        let power_ctrl = PowerControl::from_bits_truncate(buf[0]);
        Ok(power_ctrl.contains(PowerControl::AUTO_SLEEP))
    }

    /// Set whether the device has auto sleep enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_auto_sleep(&mut self, auto_sleep: bool) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_POWER_CONTROL, &mut buf)?;
        let mut power_ctrl = PowerControl::from_bits_truncate(buf[0]);
        power_ctrl.set(PowerControl::AUTO_SLEEP, auto_sleep);
        self.write_register(register::ADXL345_POWER_CONTROL, power_ctrl.bits())
    }

    /// Get whether the device is in standby mode.
    ///
    /// This is enabled by default on power up.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_standby_mode(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_POWER_CONTROL, &mut buf)?;
        let power_ctrl = PowerControl::from_bits_truncate(buf[0]);
        Ok(!power_ctrl.contains(PowerControl::MEASURE))
    }

    /// Set whether the device is in standby mode.
    ///
    /// This is enabled by default on power up.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_standby_mode(&mut self, standby: bool) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_POWER_CONTROL, &mut buf)?;
        let mut power_ctrl = PowerControl::from_bits_truncate(buf[0]);
        power_ctrl.set(PowerControl::MEASURE, !standby);
        self.write_register(register::ADXL345_POWER_CONTROL, power_ctrl.bits())
    }

    /// Get whether the device is in full resolution mode.
    ///
    /// When `true`, the output resolution changes based on the selected
    /// g-range.
    ///
    /// When `false`, the device is always in 10-bit mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_full_resolution(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_DATA_FORMAT, &mut buf)?;
        let format = DataFormat::from_bits_truncate(buf[0]);
        Ok(format.contains(DataFormat::FULL_RESOLUTION))
    }

    /// Set whether the device is in full resolution mode.
    ///
    /// When `true`, the output resolution changes based on the selected
    /// g-range.
    ///
    /// When `false`, the device is always in 10-bit mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_full_resolution(&mut self, full_res: bool) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_DATA_FORMAT, &mut buf)?;
        let mut format = DataFormat::from_bits_truncate(buf[0]);
        format.set(DataFormat::FULL_RESOLUTION, full_res);
        self.write_register(register::ADXL345_DATA_FORMAT, format.bits())
    }

    /// Get the device's measurement range.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_grange(&mut self) -> Result<GRange, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_DATA_FORMAT, &mut buf)?;
        Ok(GRange::from_byte(buf[0]))
    }

    /// Set the device's measurement range.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_grange(&mut self, range: GRange) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_DATA_FORMAT, &mut buf)?;
        let mut format = DataFormat::from_bits_truncate(buf[0]);
        format.remove(DataFormat::RANGE_MASK);
        format.insert(DataFormat::from_bits_truncate(range as u8));
        self.write_register(register::ADXL345_DATA_FORMAT, format.bits())
    }

    /// Get the device's [`FifoMode`].
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_fifo_mode(&mut self) -> Result<FifoMode, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_FIFO_CONTROL, &mut buf)?;
        Ok(FifoMode::from_byte(buf[0]))
    }

    /// Set the device's [`FifoMode`].
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_fifo_mode(&mut self, mode: FifoMode) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_FIFO_CONTROL, &mut buf)?;
        let mut control = FifoControl::from_bits_truncate(buf[0]);
        control.remove(FifoControl::FIFO_MASK);
        control.insert(FifoControl::from_bits_truncate(mode as u8));
        self.write_register(register::ADXL345_FIFO_CONTROL, control.bits())
    }

    /// Get the device's FIFO sample setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_fifo_samples(&mut self) -> Result<u8, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_FIFO_CONTROL, &mut buf)?;
        Ok(buf[0] & FifoControl::SAMPLES_MASK.bits())
    }

    /// Set the device's FIFO sample setting.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn set_fifo_samples(&mut self, samples: u8) -> Result<(), I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_FIFO_CONTROL, &mut buf)?;
        let mut control = FifoControl::from_bits_truncate(buf[0]);
        control.remove(FifoControl::SAMPLES_MASK);
        control.insert(FifoControl::from_bits_truncate(samples & FifoControl::SAMPLES_MASK.bits()));
        self.write_register(register::ADXL345_FIFO_CONTROL, control.bits())
    }

    /// Get the FIFO trigger status.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_fifo_trigger(&mut self) -> Result<bool, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_FIFO_STATUS, &mut buf)?;
        let control = FifoStatus::from_bits_truncate(buf[0]);
        Ok(control.contains(FifoStatus::TRIGGER))
    }

    /// Get the number of entries in the FIFO buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if the I2C communication fails
    pub fn get_fifo_entries(&mut self) -> Result<u8, I2C::Error> {
        let mut buf = [0u8; 1];
        self.read_register(register::ADXL345_FIFO_STATUS, &mut buf)?;
        let control = FifoStatus::from_bits_truncate(buf[0]);
        Ok(control.bits() & FifoStatus::ENTRY_MASK.bits())
    }

    /// Read data from a register
    fn read_register(&mut self, register: u8, buf: &mut [u8]) -> Result<(), I2C::Error> {
        self.i2c.write_read(self.address, core::slice::from_ref(&register), buf)
    }

    // Write data to a register
    fn write_register(&mut self, register: u8, value: u8) -> Result<(), I2C::Error> {
        self.i2c.write(self.address, [register, value].as_slice())
    }
}
