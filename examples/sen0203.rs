#![feature(addr_parse_ascii)]

mod sen0203_lib;
use sen0203_lib::*;

use esp_idf_svc::hal::prelude::Peripherals;
use std::sync::mpsc;

use esp_idf_svc::hal::delay::FreeRtos;
use log::*;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let (tx, _rx) = mpsc::channel::<f32>();

    let heartbeat_pin = peripherals.pins.gpio3;
    let sen0203_join_handle = std::thread::Builder::new()
        .stack_size(4096)
        .spawn(move || {
            let mut sen0203 = Sen0203::new(heartbeat_pin).expect("Could not initialize Sen0203");
            loop {
                if let Some(bpm) = sen0203.run() {
                    info!("\n\n\n{:?} is the bpm\n\n\n", bpm);
                    if let Err(e) = tx.send(bpm) {
                        error!("Failed to send bpm: {e}");
                        break;
                    }
                }
            }
        })?;

    sen0203_join_handle.join().unwrap();

    loop {
        FreeRtos::delay_ms(100);
    }
}
