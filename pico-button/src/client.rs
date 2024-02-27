use crate::led::{led_pattern_done, led_pattern_error, led_pattern_ping};
use cyw43::{Control, NetDriver};
use defmt::*;
use embassy_futures::select::{select, Either};
use embassy_net::{dns::DnsSocket, tcp::client::TcpClient};
use embassy_time::Timer;
use reqwless::{client::HttpClient, request::RequestBuilder, response::Status};

pub async fn sleeping_request(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    endpoint: &str,
) -> bool {
    let resource_future = client.resource(core::env!("PICOBUTTON_SERVER_ENDPOINT"));

    let mut resource = match select(resource_future, Timer::after_secs(5)).await {
        Either::First(resource) => match resource {
            Ok(resource) => resource,
            Err(err) => {
                error!(
                    "{}: Unable to connect to server endpoint: {}",
                    endpoint, err
                );

                return false;
            }
        },
        Either::Second(_) => {
            error!("{}: Timeout when connecting to server endpoint", endpoint);

            return false;
        }
    };

    let mut rx_buf = [0; 1024];

    let request_future = resource
        .post(endpoint)
        .headers(&[(
            "Authorization",
            core::concat!(
                "SharedSecret ",
                core::env!("PICOBUTTON_SERVER_SHARED_SECRET")
            ),
        )])
        .send(&mut rx_buf);

    let response = match select(request_future, Timer::after_secs(5)).await {
        Either::First(response) => match response {
            Ok(response) => response,
            Err(err) => {
                error!(
                    "{}: Unable to make request to server endpoint: {}",
                    endpoint, err
                );

                return false;
            }
        },
        Either::Second(_) => {
            error!(
                "{}: Timeout when making request to server endpoint",
                endpoint
            );

            return false;
        }
    };

    info!("{}: Server returned = {}", endpoint, response.status);

    response.status == Status::Ok
}

pub async fn sleeping_ping(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    wifi_control: &mut Control<'_>,
) {
    let did_ping = sleeping_request(client, "/api/sleeping/ping").await;

    if did_ping {
        led_pattern_ping(wifi_control).await;
    } else {
        led_pattern_error(wifi_control).await;
    }
}

pub async fn sleeping_record(
    client: &mut HttpClient<
        '_,
        TcpClient<'_, NetDriver<'_>, 4, 1024, 1024>,
        DnsSocket<'_, NetDriver<'_>>,
    >,
    wifi_control: &mut Control<'_>,
) {
    let did_record = sleeping_request(client, "/api/sleeping/record").await;

    if did_record {
        led_pattern_done(wifi_control).await;
    } else {
        led_pattern_error(wifi_control).await;
    }
}
