mod led_lib;
use led_lib::LED;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let led_pin = peripherals.pins.gpio2;
    let mut led = LED::new(led_pin).expect("could not create an LED controller");

    loop {
        led.set_led(true).expect("Can't toggle led on");
        FreeRtos::delay_ms(100);
        led.set_led(false).expect("Can't toggle led off");
        FreeRtos::delay_ms(100);
    }
}
