use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_svc::hal::delay::FreeRtos;
use mfrc522::{comm::blocking::spi::SpiInterface, Mfrc522, Uid};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");
    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;
    let sclk = pins.gpio18;
    let serial_in = pins.gpio21; // SDI
    let serial_out = pins.gpio19; // SDO
    let cs = pins.gpio5;

    // Use your HAL to create an SPI device that implements the embedded-hal `SpiDevice` trait.
    // This device manages the SPI bus and CS pin.
    let driver = SpiDriver::new(
        peripherals.spi2,
        sclk,
        serial_out,
        Some(serial_in),
        &SpiDriverConfig::new(),
    )?;
    let config = config::Config::new();
    let device = SpiDeviceDriver::new(&driver, Some(cs), &config)?;

    let itf = SpiInterface::new(device);
    let mut mfrc522 = Mfrc522::new(itf).init().unwrap();

    // The reported version is expected to be 0x91 or 0x92
    match mfrc522.version() {
        Ok(version) => log::info!("version {:x}", version),
        Err(_e) => log::error!("version error"),
    }

    loop {
        if let Ok(atqa) = mfrc522.wupa() {
            log::info!("new card detected");
            match mfrc522.select(&atqa) {
                Ok(ref _uid @ Uid::Single(ref inner)) => {
                    log::info!("card uid {:?}", inner.as_bytes());
                }
                Ok(ref _uid @ Uid::Double(ref inner)) => {
                    log::info!("card double uid {:?}", inner.as_bytes());
                }
                Ok(_) => log::info!("got other uid size"),
                Err(_e) => {
                    log::error!("Select error");
                }
            }
        }
        FreeRtos::delay_ms(1000);
    }
}
