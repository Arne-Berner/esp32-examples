//! ADC oneshot example, reading a value form a pin and printing it on the terminal
//! requires ESP-IDF v5.0 or newer

use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    use esp_idf_hal::adc::attenuation::DB_11;
    use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
    use esp_idf_hal::adc::oneshot::*;
    use esp_idf_hal::peripherals::Peripherals;

    let peripherals = Peripherals::take()?;

    let adc = AdcDriver::new(peripherals.adc1)?;

    // configuring pin to analog read, you can regulate the adc input voltage range depending on your need
    // for this example we use the attenuation of 11db which sets the input voltage range to around 0-3.6V
    let config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };

    let mut adc_pin = AdcChannelDriver::new(&adc, peripherals.pins.gpio4, &config)?;
    let mut adc_pin2 = AdcChannelDriver::new(&adc, peripherals.pins.gpio5, &config)?;

    loop {
        // you can change the sleep duration depending on how often you want to sample
        thread::sleep(Duration::from_millis(100));
        println!("ADC value: {}", adc.read(&mut adc_pin)?);
        println!("ADC value: {}", adc.read(&mut adc_pin2)?);
    }
}
