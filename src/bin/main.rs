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

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }

// Define HSV struct
struct Hsv {
    hue: u8,
    sat: u8,
    val: u8,
}

// Helper function to convert HSV to RGB
fn hsv2rgb(hsv: Hsv) -> RGB8 {
    // Basic HSV to RGB conversion
    let h = hsv.hue as f32 / 255.0;
    let s = hsv.sat as f32 / 255.0;
    let v = hsv.val as f32 / 255.0;

    if s <= 0.0 {
        return RGB8::new((v * 255.0) as u8, (v * 255.0) as u8, (v * 255.0) as u8);
    }

    let hh = (h * 6.0) % 6.0;
    let i = hh as u8;
    let ff = hh - i as f32;

    let p = v * (1.0 - s);
    let q = v * (1.0 - s * ff);
    let t = v * (1.0 - s * (1.0 - ff));

    match i {
        0 => RGB8::new((v * 255.0) as u8, (t * 255.0) as u8, (p * 255.0) as u8),
        1 => RGB8::new((q * 255.0) as u8, (v * 255.0) as u8, (p * 255.0) as u8),
        2 => RGB8::new((p * 255.0) as u8, (v * 255.0) as u8, (t * 255.0) as u8),
        3 => RGB8::new((p * 255.0) as u8, (q * 255.0) as u8, (v * 255.0) as u8),
        4 => RGB8::new((t * 255.0) as u8, (p * 255.0) as u8, (v * 255.0) as u8),
        _ => RGB8::new((v * 255.0) as u8, (p * 255.0) as u8, (q * 255.0) as u8),
    }
}

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
            let rgb = hsv2rgb(hsv);
            hue = hue.wrapping_add(10); // Increment by 10 for faster color changes

            // Display the color on the LED (brightness at 10/255)
            info!("Changing LED color");
            led.write(brightness(gamma([rgb].iter().cloned()), 255)).unwrap();
            Timer::after(Duration::from_millis(50)).await // AKA debounce for poor;
        }
    }
}