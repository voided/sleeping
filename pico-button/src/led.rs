use cyw43::Control;
use embassy_time::Timer;

const LED_PIN: u8 = 0;

pub async fn led_enable(wifi_control: &mut Control<'_>) {
    wifi_control.gpio_set(LED_PIN, true).await;
}
pub async fn led_disable(wifi_control: &mut Control<'_>) {
    wifi_control.gpio_set(LED_PIN, false).await;
}

pub async fn led_enable_millis(wifi_control: &mut Control<'_>, duration_ms: u64) {
    led_enable(wifi_control).await;
    Timer::after_millis(duration_ms).await;
    led_disable(wifi_control).await;
}

pub async fn led_pattern(
    wifi_control: &mut Control<'_>,
    led_duration: u64,
    wait_duration: u64,
    repetition: u64,
) {
    let mut counter = 0;

    while counter < repetition {
        led_enable_millis(wifi_control, led_duration).await;
        Timer::after_millis(wait_duration).await;

        counter += 1;
    }
}

pub async fn led_pattern_done(wifi_control: &mut Control<'_>) {
    led_pattern(wifi_control, 250, 250, 4).await;
}

pub async fn led_pattern_error(wifi_control: &mut Control<'_>) {
    led_pattern(wifi_control, 2000, 200, 4).await;
}

pub async fn led_pattern_ping(wifi_control: &mut Control<'_>) {
    led_pattern(wifi_control, 100, 50, 4).await;
}
