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
use embassy_time::{Duration, Instant, Timer};
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

    let mut next_ping = Instant::now();

    loop {
        // periodically hit the ping endpoint to keep things cached and connections alive
        if Instant::now() > next_ping {
            sleeping_request(&mut client, "/api/sleeping/ping").await;
            led_pattern_ping(&mut wifi_control).await;

            // set up the next ping in 10 seconds
            next_ping = Instant::now() + Duration::from_secs(10);
        }

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

            button_released(&mut client, &mut wifi_control).await;
        }
    }
}

const LED_PIN: u8 = 0;

async fn led_enable(wifi_control: &mut Control<'_>) {
    wifi_control.gpio_set(LED_PIN, true).await;
}
async fn led_disable(wifi_control: &mut Control<'_>) {
    wifi_control.gpio_set(LED_PIN, false).await;
}

async fn led_enable_millis(wifi_control: &mut Control<'_>, duration_ms: u64) {
    led_enable(wifi_control).await;
    Timer::after_millis(duration_ms).await;
    led_disable(wifi_control).await;
}

async fn led_pattern_done(wifi_control: &mut Control<'_>) {
    const LED_DURATION: u64 = 250;

    let mut counter = 0;

    while counter < 4 {
        led_enable_millis(wifi_control, LED_DURATION).await;
        Timer::after_millis(LED_DURATION).await;

        counter += 1;
    }
}

async fn led_pattern_ping(wifi_control: &mut Control<'_>) {
    const LED_DURATION: u64 = 100;

    let mut counter = 0;

    while counter < 6 {
        led_enable_millis(wifi_control, LED_DURATION).await;
        Timer::after_millis(LED_DURATION).await;

        counter += 1;
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
    sleeping_request(client, "/api/sleeping/record").await;
    led_disable(wifi_control).await;

    Timer::after_millis(1000).await;

    led_pattern_done(wifi_control).await;
}

async fn sleeping_request(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    endpoint: &str,
) {
    let mut resource = match client
        .resource(core::env!("PICOBUTTON_SERVER_ENDPOINT"))
        .await
    {
        Ok(resource) => resource,
        Err(error) => {
            error!(
                "{}: Unable to connect to server endpoint: {}",
                endpoint, error
            );
            return;
        }
    };

    let mut rx_buf = [0; 1024];

    let response = match resource
        .post(endpoint)
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
            error!(
                "{}: Unable to make request to server endpoint: {}",
                endpoint, error
            );
            return;
        }
    };

    info!("{}: Server returned = {}", endpoint, response.status);
}
