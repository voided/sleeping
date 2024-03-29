#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod client;
mod led;
mod net;
mod wifi;

use crate::client::{sleeping_ping, sleeping_record};
use crate::led::{led_disable, led_enable, led_pattern_ready};
use crate::wifi::WifiPeripherals;
use cyw43::{Control, NetDriver};
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_time::{Duration, Instant};
use reqwless::client::HttpClient;

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

    let tcp_state = TcpClientState::<4, 1024, 1024>::new();
    let tcp_client = TcpClient::new(net_stack, &tcp_state);
    let dns_socket = DnsSocket::new(net_stack);

    let mut client = HttpClient::new(&tcp_client, &dns_socket);

    // we're initialized, signal to the user
    led_pattern_ready(&mut wifi_control).await;

    let mut was_pressed = false;
    let mut next_ping = Instant::now() + Duration::from_secs(5);

    // pings should occur every 30 seconds
    const PING_DELAY: u64 = 30;

    loop {
        // periodically hit the ping endpoint to keep things cached and connections alive
        if Instant::now() >= next_ping {
            sleeping_ping(&mut client, &mut wifi_control).await;

            next_ping = Instant::now() + Duration::from_secs(PING_DELAY);
        }

        if peripherals.BOOTSEL.is_pressed() {
            // if we already detected the button was pressed, do nothing
            if was_pressed {
                continue;
            }

            // otherwise, this is the first time hitting the button
            was_pressed = true;

            // pings don't need to closely overlap with our button actions
            next_ping = Instant::now() + Duration::from_secs(PING_DELAY);

            button_pressed(&mut client, &mut wifi_control).await;
        } else {
            // button wasn't pressed, and still isn't pressed, do nothing
            if !was_pressed {
                continue;
            }

            // otherwise, this is the first time no longer pressing the button
            was_pressed = false;

            // pings don't need to closely overlap with our button actions
            next_ping = Instant::now() + Duration::from_secs(PING_DELAY);

            button_released(&mut client, &mut wifi_control).await;
        }
    }
}

#[allow(unused_variables)]
async fn button_pressed(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    wifi_control: &mut Control<'_>,
) {
    info!("Button pressed");

    led_enable(wifi_control).await;
}

async fn button_released(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    wifi_control: &mut Control<'_>,
) {
    info!("Button released");

    led_enable(wifi_control).await;
    sleeping_record(client, wifi_control).await;
    led_disable(wifi_control).await;
}
