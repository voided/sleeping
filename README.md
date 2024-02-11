# Sleep + Ping

Raspberry Pi Pico W button timestamper thing.

Basically: push button on the Pico. Pings the web server. The server records the time for my future review.

## Stack

## pico-button

- it's Rust.
- embassy-rs for embedded Rust language support.

## server

- it's C#.
- .NET8 API for the web tech.

## Stuff to know

The Pico W runs the RP2040 chip. It's a 32-bit ARM Cortex-M0+ chipset.

Wifi chip is CYW43439, the cyw43 library handles this.
