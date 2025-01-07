use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_svc::hal::delay::FreeRtos;
use tcs3472::Tcs3472;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let led = peripherals.pins.gpio4;
    let mut led_driver = PinDriver::output(led)?;
    led_driver.set_high()?;

    // can't get the interrupt to work yet. It is open drain and pulls down

    let i2c = peripherals.i2c0;
    let scl = peripherals.pins.gpio5;
    let sda = peripherals.pins.gpio6;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let driver = I2cDriver::new(i2c, sda, scl, &config)?;
    log::info!("before spawn!");

    let mut sensor = Tcs3472::new(driver);
    sensor.enable().unwrap();
    sensor.enable_rgbc().unwrap();

    /*
    sensor.set_wait_cycles(35).unwrap();
    sensor.enable_wait_long().unwrap(); // 12x mutiplicator
    sensor.enable_wait().unwrap(); // actually enable wait timer
    */

    while !sensor.is_rgbc_status_valid().unwrap() {}

    loop {
        FreeRtos::delay_ms(100);
        let _ = led_driver.toggle();

        let measurement = sensor.read_all_channels().unwrap();
        let total = measurement.red + measurement.green + measurement.blue;
        let total = f32::from(total);
        let r = f32::from(measurement.red) / total * 255.0;
        let g = f32::from(measurement.green) / total * 255.0;
        let b = f32::from(measurement.blue) / total * 255.0;

        println!(
            "Measurements: clear = {}, red = {}, green = {}, blue = {}, RGB={}, {}, {}",
            measurement.clear, measurement.red, measurement.green, measurement.blue, r, g, b
        );
        let _ = led_driver.toggle();
    }
}
