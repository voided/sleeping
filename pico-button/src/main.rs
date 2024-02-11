#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod net;
mod wifi;

use core::borrow::Borrow;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use wifi::WifiPeripherals;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = embassy_rp::init(Default::default());

    let wifi_peripherals = WifiPeripherals {
        pin_23: peripherals.PIN_23,
        pin_25: peripherals.PIN_25,
        pio0: peripherals.PIO0,
        pin_24: peripherals.PIN_24,
        pin_29: peripherals.PIN_29,
        dma_ch0: peripherals.DMA_CH0,
    };

    let (net_device, mut wifi_control) = wifi::init(spawner, wifi_peripherals).await;
    let net_stack = net::init(spawner, net_device).await;

    // if you encounter a build error here, ensure you're building with `--config .cargo/config-secrets.toml`
    let wifi_ssid = core::env!("PICOBUTTON_WIFI_SSID");
    let wifi_pass = core::env!("PICOBUTTON_WIFI_PASSWORD");

    // attempt to connect to wifi
    loop {
        match wifi_control.join_wpa2(wifi_ssid, wifi_pass).await {
            Ok(_) => break,
            Err(error) => {
                error!("failed to join wifi: {}", error.status);
            }
        }

        // delay 10 seconds between wifi connection attempts
        Timer::after(Duration::from_secs(10)).await;
    }

    // attempt to get an IP via DHCP
    info!("waiting for DHCP...");
    net_stack.wait_config_up().await;
    info!(
        "DHCP is now up, IP = {}",
        unwrap!(net_stack.config_v4()).address
    );

    loop {
        if peripherals.BOOTSEL.is_pressed() {
            info!("led on!");
            wifi_control.gpio_set(0, true).await;
        } else {
            info!("led off!");
            wifi_control.gpio_set(0, false).await;
        }
    }
}
