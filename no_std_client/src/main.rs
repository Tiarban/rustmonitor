/* 
#![no_std]
#![no_main]

use embassy_executor::Spawner; //spawner that spawns tasks
use embassy_time::{Duration, Timer}; //timekeeper for delays etc
use esp_backtrace as _; //useful for debugging
use esp_hal::timer::timg::TimerGroup; //main esp hal timer group

#[embassy_executor::task]
async fn read_current() {
    loop{
        esp_println::println!("Reading data"); //placeholder
        Timer::after(Duration::from_millis(1000)).await; //return ctrl to executor
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init(esp_hal::Config::default());

    esp_println::println!("Init!");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    spawner.spawn(read_current()).ok();

    loop {
        esp_println::println!("Bing!");
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
*/
#![no_std] //using embassy instead of std libraries
#![no_main] //not using fn main, async main instead
//#![feature(type_alias_impl_trait)] //simplifies fn signatures for async

//use std::{thread::sleep, time::Duration, sync::{Mutex, Arc}, str::from_utf8, collections::HashMap};
//using embassy time now instead of std

//use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
//use heapless::String; //for correct ssid & password data conversion

use embassy_executor::Spawner; //embassy's executor's spawner
use embassy_time::{Duration, Timer}; //now duration and timer is from embassy time
use esp_backtrace as _; //as _ imported but not used
use heapless::{String, Vec};

use core::net::Ipv4Addr;

use embassy_net::{
    Config,
    tcp::TcpSocket,
    IpListenEndpoint,
    Ipv4Cidr,
    Runner,
    StackResources,
    StaticConfigV4,
};
use esp_alloc as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::{print, println};
use esp_wifi::{
    init,
    wifi::{
        ClientConfiguration,
        Configuration,
        WifiController,
        WifiEvent,
        WifiDevice,
        WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};
use smoltcp::wire::Ipv4Address;


macro_rules! mk_static { //alternative to normal static cell for persistent storage (like nvs from esp-idf-svc)
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}


#[embassy_executor::task]
async fn read_current () { //for reading data
    loop{
        esp_println::println!("Reading data"); //placeholder
        Timer::after(Duration::from_millis(1000)).await; //return ctrl to executor
    }
}

#[embassy_executor::task]
async fn sta_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}

//no task -> needs to be synchronous as this function must happen before net tasks
async fn setup_wifi (ssid: String<32>, password: String<64>, mut wifi: WifiController<'static> ) { //static extends lifetime - compiler complains
     //Wifi configuration

    /*pub enum Configuration {
        None,
        Client(ClientConfiguration),
        AccessPoint(AccessPointConfiguration),
        Mixed(ClientConfiguration, AccessPointConfiguration),
    }*/ // this enum is set for client, ap or mixed configuration.
    
    /*pub struct ClientConfiguration {
        pub ssid: String<32>,
        pub bssid: Option<[u8; 6]>,
        pub auth_method: AuthMethod,
        pub password: String<64>,
        pub channel: Option<u8>,
    }*/ //this struct is how the wifi options are configured for the client, which is the purpose of this code
    
    wifi.set_configuration(&Configuration::Client(ClientConfiguration { //configuration for client
        ssid,
        password, //explicit defining done above - examples for using .into() or from here dont work
                    //could be used later for WPA3 implementation
        ..Default::default() //fills non specified methods with default config
    })).unwrap(); //note these brackets are closed here for the nested struture of accessing the clientconfig 

    wifi.start_async().await.unwrap(); //start_async made in esp-wifi for async ops

    wifi.connect_async().await.unwrap(); //self explanatory lines

    while !wifi.is_connected().unwrap() {
        // Get and print connetion configuration
        let config = wifi.configuration().unwrap();
        esp_println::println!("Waiting for station {:?}", config);

        Timer::after(Duration::from_millis(5000)).await; //added non blocking delay before checking again if wifi is connected 
    }
    
    esp_println::println!("Connected" ); //boilerplate for checking connection
    
} 

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) { 

    esp_println::logger::init_logger_from_env();

    esp_println::println!("Init!");

    //handles:

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max()); //sets clock to max clock speed
    let peripherals = esp_hal::init(config);//get peripheral using esp_hal abstraction

    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let timg1 = TimerGroup::new(peripherals.TIMG1);
        esp_hal_embassy::init(timg1.timer0);

    let mut rng = Rng::new(peripherals.RNG); // Random Number Generator

    let init = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap() //switched rng.clone to into, not sure the consequences
    );

    let wifi = peripherals.WIFI;

    let (_wifi_ap_interface, wifi_sta_interface, mut controller) =
        esp_wifi::wifi::new_ap_sta(&init, wifi).unwrap();

    

     /*pub fn new<M: WifiModemPeripheral>(
        modem: impl Peripheral<P = M> + 'd,
        sysloop: EspSystemEventLoop,
        nvs: Option<EspDefaultNvsPartition>
    ) -> Result<Self, EspError> */ // this is the signature of the method to get wifi driver handle

    let mut ssid = String::<32>::new();
    ssid.push_str("Three_E42796").unwrap(); //explicit type required for config for both 

    let mut password = String::<64>::new();
    password.push_str("2hG{w?24").unwrap(); 

    setup_wifi(ssid, password, controller).await;

    let mut dns_servers = Vec::<Ipv4Address, 3>::new();
    dns_servers.push(Ipv4Address::new(8, 8, 8, 8)).unwrap();

    let sta_config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 18, 50), 24), // Static IP
        gateway: Some(Ipv4Address::new(192, 168, 18, 1)), // Router IP
        dns_servers, // Google DNS
    });

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;


    let (sta_stack, sta_runner) = embassy_net::new(
        wifi_sta_interface,
        sta_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(read_current()).ok();
    spawner.spawn(sta_task(sta_runner)).ok();

esp_println::println!("📡 MAC Address: {:?}", mac);
 //returns Result type if it's ok - no errors detected

 loop {
    if let Some(ip_config) = sta_stack.config_v4() {
        esp_println::println!("IP Address: {:?}", ip_config.address);
        break;
    } else {
        esp_println::println!("Since DHCP is disabled here, IP address failed to be assigned");
    }
    Timer::after(Duration::from_secs(1)).await;
}

loop {
    Timer::after(Duration::from_secs(5)).await;
}
}

/* 
fn main() -> anyhow::Result<()> { //anyhow::result is return type leveraging error control
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    //handles:

    let peripherals = Peripherals::take().unwrap(); //get peripheral handle

    let sysloop = EspSystemEventLoop::take()?; //creates system event loop associated with wifi
    let nvs = EspDefaultNvsPartition::take()?; //non volatile partition to store needed data

     /*pub fn new<M: WifiModemPeripheral>(
        modem: impl Peripheral<P = M> + 'd,
        sysloop: EspSystemEventLoop,
        nvs: Option<EspDefaultNvsPartition>
    ) -> Result<Self, EspError> */ // this is the signature of the method to get wifi driver handle

    let wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?; // i/p to 'new' method on line 26
    //wifi mut is now the wifi handle driver (like peripherals). <'_> means that lifetime is inferred from stuff inside

    let mut ssid = String::<32>::new();
    ssid.push_str("Three_E42796").unwrap(); //explicit type required for config for both 

    let mut password = String::<64>::new();
    password.push_str("2hG{w?24").unwrap(); 

    setup_wifi(ssid, password, wifi);


    spawner.spawn(read_current()).ok();
    //returns Result type if it's ok - no errors detected
    Ok(())
}*/