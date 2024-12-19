#![feature(addr_parse_ascii)]

mod osc_lib;
use osc_lib::Osc;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{modem::Modem, prelude::Peripherals},
    nvs::EspDefaultNvsPartition,
    wifi::{AccessPointConfiguration, AuthMethod, BlockingWifi, Configuration, EspWifi},
};
use std::net::*;

use log::*;

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASS: &str = env!("WIFI_PASS");
const OSC_PORT: &str = env!("OSC_PORT");
const OSC_IP: &str = env!("OSC_IP");

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let wifi = create_wifi(&sys_loop, &nvs, peripherals.modem)?;
    let ip = wifi.wifi().sta_netif().get_ip_info()?.ip;

    let port = OSC_PORT.parse::<u16>().unwrap();

    info!("before");
    info!("after");
    // Create thread to receive/send OSC
    // Larger stack size is required (default is 3 KB)
    // You can customize default value by CONFIG_ESP_SYSTEM_EVENT_TASK_STACK_SIZE in sdkconfig
    let osc_join_handle = std::thread::Builder::new()
        .stack_size(8192)
        .spawn(move || {
            let mut osc = Osc::new(ip, port);
            loop {
                let ip_in_bytes = OSC_IP.as_bytes();
                let ip = Ipv4Addr::parse_ascii(ip_in_bytes).expect("could not convert it to ipv4");
                let value = 22.3;
                let value = rosc::OscType::Float(value);
                let addr = SocketAddr::new(IpAddr::V4(ip), port);
                if let Err(e) = osc.run(addr, "/topic/name", value) {
                    error!("Failed to run OSC: {e}");
                    break;
                }
            }
        })?;

    osc_join_handle.join().unwrap();
    core::mem::forget(wifi);

    Ok(())
}

fn create_wifi(
    sys_loop: &EspSystemEventLoop,
    nvs: &EspDefaultNvsPartition,
    modem: Modem,
) -> Result<BlockingWifi<EspWifi<'static>>, EspError> {
    let esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs.clone()))?;
    let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop.clone())?;

    let config = Configuration::AccessPoint(AccessPointConfiguration {
        ssid: WIFI_SSID.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        password: WIFI_PASS.try_into().unwrap(),
        ..Default::default()
    });

    wifi.set_configuration(&config)?;

    wifi.start()?;

    // Wait until the network interface is up
    wifi.wait_netif_up()?;

    while !wifi.is_up().unwrap() {
        // Get and print connection configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting to set up {:?}", config);
    }

    Ok(wifi)
}
