#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_println as _;
use smart_leds::{RGB8, SmartLedsWrite, brightness, gamma};

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Pull},
    rmt::Rmt,
    time::Rate,
    timer::systimer::SystemTimer,
};
use esp_hal_smartled::{smartLedBuffer, SmartLedsAdapter};
use smart_leds::hsv::{hsv2rgb, Hsv};

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

    // Set up RMT @80 MHz for NeoPixel timing
    let rmt = Rmt::new(per.RMT, Rate::from_mhz(80)).unwrap();

    // On-board WS2812 on GPIO8, one LED
    let buffer = smartLedBuffer!(1); // Creates appropriate buffer for 1 LED

    // Create SmartLedsAdapter directly - let it handle pin configuration
    let mut led = SmartLedsAdapter::new(rmt.channel0, per.GPIO8, buffer);

    info!("Hold BOOT to toggle LED color");

    // Initialize with LED off
    led.write(brightness([RGB8::new(0, 0, 0)].iter().cloned(), 0)).unwrap();

    // For rainbow effect
    let mut hue: u8 = 0;

    loop {
        if button.is_low() {
            info!("Button pressed!");

            // Build HSV and convert to RGB8
            let hsv = Hsv { hue, sat: 255, val: 255 };
            let rgb: RGB8 = hsv2rgb(Hsv { hue, sat: 255, val: 255 });
            hue = hue.wrapping_add(10); // Increment hue for next color

            // Display the color on the LED (brightness at 10/255)
            info!("Changing LED color");
            if let Err(e) = led.write(brightness(gamma([rgb].iter().cloned()), 10)) {
                info!("LED write error");
            }
            Timer::after(Duration::from_millis(50)).await // AKA debounce for poor;
        }
    }
}