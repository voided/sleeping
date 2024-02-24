#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod net;
mod wifi;

use cyw43::{Control, NetDriver};
use defmt::*;
use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use reqwless::{client::HttpClient, request::RequestBuilder};
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

    let mut was_pressed = false;

    let tcp_state = TcpClientState::<4, 1024, 1024>::new();
    let tcp_client = TcpClient::new(net_stack, &tcp_state);
    let dns_socket = DnsSocket::new(net_stack);

    let mut client = HttpClient::new(&tcp_client, &dns_socket);

    loop {
        if peripherals.BOOTSEL.is_pressed() {
            // if we already detected the button was pressed, do nothing
            if was_pressed {
                continue;
            }

            // otherwise, this is the first time hitting the button
            was_pressed = true;

            button_pressed(&mut client, &mut wifi_control).await;
        } else {
            // button wasn't pressed, and still isn't pressed, do nothing
            if !was_pressed {
                continue;
            }

            // otherwise, this is the first time no longer pressing the button
            was_pressed = false;

            button_released(&mut wifi_control).await;
        }
    }
}

async fn button_pressed(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    wifi_control: &mut Control<'_>,
) {
    info!("button!");

    let mut resource = match client
        .resource(core::env!("PICOBUTTON_SERVER_ENDPOINT"))
        .await
    {
        Ok(resource) => resource,
        Err(error) => {
            error!("Unable to connect to server endpoint: {}", error);
            return;
        }
    };

    let mut rx_buf = [0; 4096];

    let response = match resource
        .post("/api/ping")
        .headers(&[(
            "Authorization",
            core::concat!(
                "SharedSecret ",
                core::env!("PICOBUTTON_SERVER_SHARED_SECRET")
            ),
        )])
        .send(&mut rx_buf)
        .await
    {
        Ok(response) => response,
        Err(error) => {
            error!("Unable to make request to server endpoint: {}", error);
            return;
        }
    };

    info!("HTTP status = {}", response.status);

    for (header, value) in response.headers() {
        info!(
            "Header {} = {}",
            header,
            core::str::from_utf8(value).unwrap()
        );
    }

    wifi_control.gpio_set(0, true).await;
}

async fn button_released(wifi_control: &mut Control<'_>) {
    wifi_control.gpio_set(0, false).await;
}
