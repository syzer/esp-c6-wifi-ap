#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use esp_hal::{
    clock::CpuClock,
    gpio::{Input, InputConfig, Pull},
    rmt::Rmt,
    time::Rate,
    timer::systimer::SystemTimer,
};
use ble_wifi::Led;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let cfg = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let per = esp_hal::init(cfg);
    esp_alloc::heap_allocator!(size: 72 * 1024);
    let t0 = SystemTimer::new(per.SYSTIMER);
    esp_hal_embassy::init(t0.alarm0);

    // BOOT button on GPIO9
    let button = Input::new(per.GPIO9, InputConfig::default().with_pull(Pull::Up));

    // RMT @80 MHz
    let rmt = Rmt::new(per.RMT, Rate::from_mhz(80)).unwrap();

    // Create our single‐LED driver
    let mut led = Led::new(rmt.channel0, per.GPIO8);

    esp_println::println!("Hold BOOT to change LED color");

    loop {
        if button.is_low() {
            if led.random_color(20).is_err() {
                esp_println::println!("LED write error");
            }
            Timer::after(Duration::from_millis(50)).await;
        }
        Timer::after(Duration::from_millis(20)).await;
    }
}
