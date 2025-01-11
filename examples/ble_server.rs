use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::prelude::*;
use std::sync::mpsc;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    println!("Configuring output channel");

    let mut channel = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &config::TimerConfig::new().frequency(25.kHz().into()),
        )?,
        peripherals.pins.gpio4,
    )?;

    println!("Starting duty-cycle loop");

    let max_duty = channel.get_max_duty();

    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();

    let server = ble_device.get_server();
    server.on_connect(|server, desc| {
        ::log::info!("Client connected: {:?}", desc);

        server
            .update_conn_params(desc.conn_handle(), 24, 48, 0, 60)
            .unwrap();

        if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
            ::log::info!("Multi-connect support: start advertising");
            ble_advertising.lock().start().unwrap();
        }
    });

    server.on_disconnect(|_desc, reason| {
        ::log::info!("Client disconnected ({:?})", reason);
    });

    let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

    // A static characteristic.
    let static_characteristic = service.lock().create_characteristic(
        uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
        NimbleProperties::READ,
    );
    static_characteristic
        .lock()
        .set_value("Hello, world!".as_bytes());

    // A characteristic that notifies every second.
    let notifying_characteristic = service.lock().create_characteristic(
        uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    notifying_characteristic.lock().set_value(b"Initial value.");

    let (tx, rx) = mpsc::channel::<u8>();

    // A writable characteristic.
    let writable_characteristic = service.lock().create_characteristic(
        uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
        NimbleProperties::READ | NimbleProperties::WRITE,
    );
    writable_characteristic
        .lock()
        .on_read(move |_, _| {
            ::log::info!("Read from writable characteristic.");
        })
        .on_write(move |args| {
            ::log::info!(
                "Wrote to writable characteristic: {:?} -> {:?}",
                args.current_data(),
                args.recv_data()
            );
            tx.send(args.recv_data()[0]).unwrap();
        });

    ble_advertising.lock().set_data(
        BLEAdvertisementData::new()
            .name("ESP32-GATT-Server")
            .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa")),
    )?;
    ble_advertising.lock().start()?;

    server.ble_gatts_show_local();

    let mut _counter = 0;
    loop {
        if let Ok(received) = rx.recv_timeout(Duration::from_millis(200)) {
            ::log::info!("this is received: {:?}", received);
            let duty_cycle = max_duty * received as u32 / 255u32;
            ::log::info!("this is max: {:?}\nduty: {:?}", max_duty, duty_cycle);

            channel.set_duty(duty_cycle as u32)?;
        }
    }
}
