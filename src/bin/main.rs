#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_println as _;
use smart_leds::{RGB8, SmartLedsWrite};

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Pull, Output, Level, OutputConfig},
    rmt::{Rmt, TxChannelConfig},
    time::Rate,
    timer::systimer::SystemTimer,
};
use esp_hal_smartled::SmartLedsAdapter;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let cfg = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let per = esp_hal::init(cfg);
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let t0 = SystemTimer::new(per.SYSTIMER);
    esp_hal_embassy::init(t0.alarm0);

    // BOOT button on GPIO9
    let button = Input::new(
        per.GPIO9,
        InputConfig::default().with_pull(Pull::Up),
    );

    // Onboard LED on GPIO8 for ESP32-C6
    let mut onboard_led = Output::new(
        per.GPIO8,
        Level::Low,
        OutputConfig::default(),
    );

    info!("Hold BOOT to toggle onboard LED");

    loop {
        if button.is_low() {
            info!("Button pressed!");
            onboard_led.set_high();
            Timer::after(Duration::from_millis(500)).await;
            onboard_led.set_low();
        }
        Timer::after(Duration::from_millis(50)).await;
    }
}