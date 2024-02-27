use cyw43::{Control, NetDriver};
use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_24, PIN_25, PIN_29, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;

const WIFI_FIRMWARE: &[u8] = include_bytes!("../firmware/43439A0.bin");
const WIFI_CLM: &[u8] = include_bytes!("../firmware/43439A0_clm.bin");

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

static CYW43_STATE: StaticCell<cyw43::State> = StaticCell::new();

pub struct WifiPeripherals {
    pub pin_23: PIN_23,
    pub pin_25: PIN_25,
    pub pio0: PIO0,
    pub pin_24: PIN_24,
    pub pin_29: PIN_29,
    pub dma_ch0: DMA_CH0,
}

/// Initialize the wifi chip and return the control handle.
pub async fn init<'a>(
    spawner: Spawner,
    peripherals: WifiPeripherals,
) -> (NetDriver<'a>, Control<'a>) {
    // GPIO23 - OP wireless power on signal
    let pwr = Output::new(peripherals.pin_23, Level::Low);
    // GPIO25 - OP wireless SPI chip select
    let cs = Output::new(peripherals.pin_25, Level::High);

    // initialize one of the PIO blocks, as its used to implement SPI with the wifi chip
    let mut pio = Pio::new(peripherals.pio0, Irqs);

    // and now initialize the non-standard RP pico SPI driver
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        peripherals.pin_24,
        peripherals.pin_29,
        peripherals.dma_ch0,
    );

    let state = CYW43_STATE.init(cyw43::State::new());

    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, WIFI_FIRMWARE).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(WIFI_CLM).await;

    control
        .set_power_management(cyw43::PowerManagementMode::Performance)
        .await;

    // if you encounter a build error here, ensure you're building with `--config .cargo/config-secrets.toml`
    let wifi_ssid = core::env!("PICOBUTTON_WIFI_SSID");
    let wifi_pass = core::env!("PICOBUTTON_WIFI_PASSWORD");

    // attempt to connect to wifi
    loop {
        match control.join_wpa2(wifi_ssid, wifi_pass).await {
            Ok(_) => break,
            Err(error) => {
                error!("failed to join wifi: {}", error.status);
            }
        }

        // delay 10 seconds between wifi connection attempts
        Timer::after(Duration::from_secs(10)).await;
    }

    (net_device, control)
}
