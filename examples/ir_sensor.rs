//! ADC oneshot example, reading a value form a pin and printing it on the terminal
//! requires ESP-IDF v5.0 or newer

use esp_idf_hal::adc::attenuation::{DB_11, DB_2_5, DB_6, NONE};
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::*;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use std::thread;
use std::time::Duration;

const AVERAGE_COUNT: usize = 10;

fn main() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;

    // IR Emitter
    let led_pin1 = peripherals.pins.gpio3;
    let led_pin2 = peripherals.pins.gpio6;
    let led_pin3 = peripherals.pins.gpio9;
    let led_pin4 = peripherals.pins.gpio12;
    let led_pin5 = peripherals.pins.gpio15;
    let mut led1 = PinDriver::output(led_pin1)?;
    let mut led2 = PinDriver::output(led_pin2)?;
    let mut led3 = PinDriver::output(led_pin3)?;
    let mut led4 = PinDriver::output(led_pin4)?;
    let mut led5 = PinDriver::output(led_pin5)?;
    led1.set_high()?;
    led2.set_high()?;
    led3.set_high()?;
    led4.set_high()?;
    led5.set_high()?;

    let adc1 = AdcDriver::new(peripherals.adc1)?;
    let adc2 = AdcDriver::new(peripherals.adc2)?;

    let config1 = AdcChannelConfig {
        attenuation: DB_6,
        ..Default::default()
    };
    let config2 = AdcChannelConfig {
        attenuation: DB_2_5,
        ..Default::default()
    };

    let mut adc_pin1 = AdcChannelDriver::new(&adc1, peripherals.pins.gpio2, &config1)?;
    let mut adc_pin2 = AdcChannelDriver::new(&adc1, peripherals.pins.gpio5, &config1)?;
    let mut adc_pin3 = AdcChannelDriver::new(&adc1, peripherals.pins.gpio8, &config1)?;
    let mut adc_pin4 = AdcChannelDriver::new(&adc2, peripherals.pins.gpio11, &config2)?;
    let mut adc_pin5 = AdcChannelDriver::new(&adc2, peripherals.pins.gpio14, &config2)?;

    // get the average of the last 10 readings, then the sleep duration can be 0 or faster
    loop {
        let mut readings_sum = 0;
        for _ in 0..AVERAGE_COUNT {
            readings_sum += adc1.read(&mut adc_pin1)?;
            // println!("ADC value: {}", adc.read(&mut adc_pin)?);
        }
        // println!("Reading sum: {}", readings_sum);

        let reading_average = readings_sum as f32 / AVERAGE_COUNT as f32;

        println!("1ADC value: {}", reading_average);
        thread::sleep(Duration::from_millis(100));

        let mut readings_sum = 0;
        for _ in 0..AVERAGE_COUNT {
            readings_sum += adc1.read(&mut adc_pin2)?;
            // println!("ADC value: {}", adc.read(&mut adc_pin)?);
        }
        // println!("Reading sum: {}", readings_sum);

        let reading_average = readings_sum as f32 / AVERAGE_COUNT as f32;

        println!("2ADC value: {}", reading_average);
        thread::sleep(Duration::from_millis(100));

        let mut readings_sum = 0;
        for _ in 0..AVERAGE_COUNT {
            readings_sum += adc1.read(&mut adc_pin3)?;
            // println!("ADC value: {}", adc.read(&mut adc_pin)?);
        }
        // println!("Reading sum: {}", readings_sum);

        let reading_average = readings_sum as f32 / AVERAGE_COUNT as f32;

        println!("3ADC value: {}", reading_average);
        thread::sleep(Duration::from_millis(100));

        let mut readings_sum = 0;
        for _ in 0..AVERAGE_COUNT {
            readings_sum += adc2.read(&mut adc_pin4)?;
            // println!("ADC value: {}", adc.read(&mut adc_pin)?);
        }
        // println!("Reading sum: {}", readings_sum);

        let reading_average = readings_sum as f32 / AVERAGE_COUNT as f32;

        println!("4ADC value: {}", reading_average);
        thread::sleep(Duration::from_millis(100));

        let mut readings_sum = 0;
        for _ in 0..AVERAGE_COUNT {
            readings_sum += adc2.read(&mut adc_pin5)?;
            // println!("ADC value: {}", adc.read(&mut adc_pin)?);
        }
        // println!("Reading sum: {}", readings_sum);

        let reading_average = readings_sum as f32 / AVERAGE_COUNT as f32;

        println!("5ADC value: {}", reading_average);
        thread::sleep(Duration::from_millis(100));
    }
}
