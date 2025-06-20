#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::{Spawner, task};
use embassy_time::{Timer, Duration};
use static_cell::StaticCell;

use esp_hal::{
    clock::CpuClock,
    peripherals,
    timer::systimer::SystemTimer,
};
use log::{info, LevelFilter};

use esp_wifi::wifi::{
    self, initialize, WifiInitFor,
    AccessPointConfiguration, ClientConfiguration, Configuration,
    WifiDevice, WifiDeviceMode,
};

use embassy_net::{Stack, Config as NetConfig, StackResources};

/// STA credentials
const ST_SSID: &str = "YourHomeWiFi";
const ST_PASS: &str = "YourPassword";

/// Soft-AP credentials
const AP_SSID: &str = "RustyAP";
const AP_PASS: &str = "supersafe";

static DEVICE: StaticCell<WifiDevice<'static>> = StaticCell::new();
static STACK : StaticCell<Stack<'static>> = StaticCell::new();
static mut RESOURCES: StackResources<4> = StackResources::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // ── HAL / Embassy boilerplate ────────────────────────────────────────────
    let cfg = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let per = esp_hal::init(cfg);
    esp_alloc::heap_allocator!(size: 72 * 1024);

    let systimer = SystemTimer::new(per.SYSTIMER);
    esp_hal_embassy::init(systimer.alarm0);

    esp_println::logger::init_logger(LevelFilter::Info);
    info!("booting…");

    // ── esp-wifi driver init ────────────────────────────────────────────────
    let init = initialize(
        systimer.alarm1,
        unsafe { peripherals::RNG::steal() },
        WifiInitFor::Wifi,
    )
    .expect("wifi init");

    // Mixed (AP+STA)
    let (mut ctrl, ifaces) =
        wifi::new(&init, WifiDeviceMode::StaAp).expect("new()");
    let sta_iface = ifaces.sta;

    // Configure Soft-AP
    let ap_cfg = AccessPointConfiguration {
        ssid:      AP_SSID.try_into().unwrap(),
        password:  AP_PASS.try_into().unwrap(),
        channel:   6,
        ..Default::default()
    };

    // Configure STA
    let sta_cfg = ClientConfiguration {
        ssid:     ST_SSID.try_into().unwrap(),
        password: ST_PASS.try_into().unwrap(),
        ..Default::default()
    };

    ctrl.set_configuration(&Configuration::Mixed(sta_cfg, ap_cfg))
        .expect("set config");
    ctrl.start().await.expect("wifi start");
    ctrl.connect().await.expect("wifi connect");

    info!("Soft-AP \"{}\" up; STA connecting to \"{}\"…", AP_SSID, ST_SSID);

    // ── Network stack on the STA side (DHCP) ────────────────────────────────
    let device = DEVICE.init(WifiDevice::wrap(sta_iface));
    let stack = STACK.init(Stack::new(
        device,
        unsafe { &mut RESOURCES },
        NetConfig::dhcpv4(Default::default()),
        0x1234_5678,
    ));

    spawner.spawn(net_task(stack)).unwrap();

    // ── Blink LED every 2 s just to prove it's alive ────────────────────────
    loop {
        info!("STA connected: {}", ctrl.is_connected());
        Timer::after(Duration::from_secs(2)).await;
    }
}

#[task]
async fn net_task(stack: &'static Stack<'static>) {
    stack.run().await;
}