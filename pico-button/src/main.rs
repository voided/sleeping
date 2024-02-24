#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod net;
mod wifi;

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use wifi::WifiPeripherals;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut peripherals = embassy_rp::init(Default::default());

    // initialize the wifi chip
    let (net_device, mut wifi_control) = wifi::init(
        spawner,
        WifiPeripherals {
            pin_23: peripherals.PIN_23,
            pin_25: peripherals.PIN_25,
            pio0: peripherals.PIO0,
            pin_24: peripherals.PIN_24,
            pin_29: peripherals.PIN_29,
            dma_ch0: peripherals.DMA_CH0,
        },
    )
    .await;

    // initialize the network software stack
    let net_stack = net::init(spawner, net_device).await;



        // delay 10 seconds between wifi connection attempts
        Timer::after(Duration::from_secs(10)).await;
    }

    let mut wasPressed = false;

    loop {
        if peripherals.BOOTSEL.is_pressed() {
            // if we already detected the button was pressed, do nothing
            if wasPressed {
                continue;
            }

            // otherwise, this is the first time hitting the button
            wasPressed = true;

            info!("button!");
            wifi_control.gpio_set(0, true).await;
        } else {
            // button wasn't pressed, and still isn't pressed, do nothing
            if !wasPressed {
                continue;
            }

            // otherwise, this is the first time no longer pressing the button
            wasPressed = false;

            wifi_control.gpio_set(0, false).await;
        }
    }
}
