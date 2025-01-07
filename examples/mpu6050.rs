use esp_idf_hal::delay::Delay;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use mpu6050_dmp::{
    address::Address, quaternion::Quaternion, sensor::Mpu6050, yaw_pitch_roll::YawPitchRoll,
};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let i2c = peripherals.i2c0;
    let scl = peripherals.pins.gpio17;
    let sda = peripherals.pins.gpio18;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let driver = I2cDriver::new(i2c, sda, scl, &config)?;

    log::info!("before spawn!");
    let mut sensor = Mpu6050::new(driver, Address::default()).unwrap();
    log::info!("spawned!");
    let mut delay = Delay::new_default();

    sensor.initialize_dmp(&mut delay).unwrap();
    log::info!("initialized!");

    loop {
        match sensor.get_fifo_count() {
            Ok(len) => {
                if len >= 28 {
                    let mut buf = [0; 28];
                    let buf = sensor.read_fifo(&mut buf).unwrap();
                    let quat = Quaternion::from_bytes(&buf[..16]).unwrap();
                    let ypr = YawPitchRoll::from(quat);
                    log::info!("{:.5?}; {:.5?}; {:.5?};", ypr.yaw, ypr.pitch, ypr.roll);
                }
            }
            Err(_) => sensor.initialize_dmp(&mut delay).unwrap(),
        }
    }
}
