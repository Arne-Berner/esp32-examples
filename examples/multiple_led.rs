use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let led_pin1 = peripherals.pins.gpio3;
    let led_pin2 = peripherals.pins.gpio6;
    let led_pin3 = peripherals.pins.gpio9;
    let led_pin4 = peripherals.pins.gpio12;
    let led_pin5 = peripherals.pins.gpio15;
    let led_pin6 = peripherals.pins.gpio17;
    let led_pin7 = peripherals.pins.gpio18;
    let mut led1 = PinDriver::output(led_pin1)?;
    let mut led2 = PinDriver::output(led_pin2)?;
    let mut led3 = PinDriver::output(led_pin3)?;
    let mut led4 = PinDriver::output(led_pin4)?;
    let mut led5 = PinDriver::output(led_pin5)?;
    let mut led6 = PinDriver::output(led_pin6)?;
    let mut led7 = PinDriver::output(led_pin7)?;

    loop {
        led1.set_high()?;
        led2.set_high()?;
        led3.set_high()?;
        led4.set_high()?;
        led5.set_high()?;
        led6.set_high()?;
        led7.set_high()?;
        FreeRtos::delay_ms(2000);
        led1.set_low()?;
        led2.set_low()?;
        led3.set_low()?;
        led4.set_low()?;
        led5.set_low()?;
        led6.set_low()?;
        led7.set_low()?;
        FreeRtos::delay_ms(2000);
    }
}
