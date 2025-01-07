use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_sys as _;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // Take ownership of device
    let ble_device = BLEDevice::take();

    // Obtain handle for peripheral advertiser
    let ble_advertiser = ble_device.get_advertising();

    // Obtain handle for server
    let server = ble_device.get_server();

    // Define server connect behaviour
    server.on_connect(|server, clntdesc| {
        // Print connected client data
        println!("{:?}", clntdesc);
        // Update connection parameters
        server
            .update_conn_params(clntdesc.conn_handle(), 24, 48, 0, 60)
            .unwrap();
    });

    // Define server disconnect behaviour
    server.on_disconnect(|_desc, _reason| {
        println!("Disconnected, back to advertising");
    });

    // Create a service with custom UUID
    let my_service = server.create_service(uuid128!("9b574847-f706-436c-bed7-fc01eb0965c1"));

    // Create a characteristic to associate with created service
    let my_service_characteristic = my_service.lock().create_characteristic(
        uuid128!("5161858c-3387-4cff-a980-57c81ebc561d"),
        NimbleProperties::READ | NimbleProperties::NOTIFY,
    );
    let write_characteristic = my_service.lock().create_characteristic(
        uuid128!("35ea1845-a8ca-441c-9317-df111a189a17"),
        NimbleProperties::READ | NimbleProperties::WRITE,
    );
    write_characteristic.lock().on_write(move |args| {
        ::log::info!(
            "Wrote to writable characteristic: {:?} -> {:?}",
            args.current_data(),
            args.recv_data(),
        );
    });

    // Modify characteristic value
    my_service_characteristic.lock().set_value(b"Start Value");

    // Configure Advertiser Data
    ble_advertiser
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("ESP32 Server")
                .add_service_uuid(uuid128!("9b574847-f706-436c-bed7-fc01eb0965c1")),
        )
        .unwrap();

    // Start Advertising
    ble_advertiser.lock().start().unwrap();

    // (Optional) Print dump of local GATT table
    // server.ble_gatts_show_local();

    // Init a value to pass to characteristic
    let mut val = 0;

    loop {
        FreeRtos::delay_ms(1000);
        let temp = write_characteristic.lock().value_mut().value();
        if !temp.is_empty() {
            log::info!("IT WORKS!");
            val = write_characteristic.lock().value_mut().value()[0];
            //log::info!("received: {:?}", val);

            for i in 0..write_characteristic.lock().value_mut().len() {
                log::info!(
                    "received: {:?}",
                    write_characteristic.lock().value_mut().value()[i]
                );
            }
            write_characteristic.lock().set_value(&[]);
        }
        my_service_characteristic.lock().set_value(&[val]).notify();
        val = val.wrapping_add(1);
    }
}
