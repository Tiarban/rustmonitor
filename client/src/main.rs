#![no_std] //using embassy instead of std libraries
#![no_main] //not using fn main, async main instead
#![feature(type_alias_impl_trait)] //simplifies fn signatures for async

//use std::{thread::sleep, time::Duration, sync::{Mutex, Arc}, str::from_utf8, collections::HashMap};
//using embassy time now instead of std

use anyhow::{self, Error};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use heapless::String; //for correct ssid & password data conversion

use embassy_executor::Spawner; //embassy's executor's spawner
use embassy_time::{Duration, Timer}; //now duration and timer is from embassy time
use esp_backtrace as _; //as _ imported but not used
use esp_hal::{
    clock::ClockControl,
    embassy::{self},
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
}; //using esp-hal for this stuff rather than std

#[embassy_executor::task]
async fn read_current () { //for reading data
    loop{
        esp_println!("Reading data") //placeholder
        Time::after(Duration::from_millis(1000)).await; //return ctrl to executor
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> anyhow::Result<()> { //anyhow::result is return type leveraging error control
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Hello, world!");

    //handles:

    let peripherals = Peripherals::take().unwrap(); //get peripheral handle

    let sysloop = EspSystemEventLoop::take()?; //creates system event loop associated with wifi
    let nvs = EspDefaultNvsPartition::take()?; //non volatile partition to store needed data

     /*pub fn new<M: WifiModemPeripheral>(
        modem: impl Peripheral<P = M> + 'd,
        sysloop: EspSystemEventLoop,
        nvs: Option<EspDefaultNvsPartition>
    ) -> Result<Self, EspError> */ // this is the signature of the method to get wifi driver handle

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?; // i/p to 'new' method on line 26
    //wifi mut is now the wifi handle driver (like peripherals)

    let mut ssid = String::<32>::new();
    ssid.push_str("Three_E42796").unwrap(); //explicit type required for config for both 

    let mut password = String::<64>::new();
    password.push_str("2hG{w?24").unwrap(); 

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
        auth_method: AuthMethod::None, //could be used later for WPA3 implementation
        ..Default::default() //fills non specified methods with default config
    }))?; //note these brackets are closed here for the nested struture of accessing the clientconfig 

    wifi.start().await?;

    wifi.connect().await?; //self explanatory lines

    while !wifi.is_connected().unwrap() {
        // Get and print connetion configuration
        let config = wifi.get_configuration().unwrap();
        esp_println!("Waiting for station {:?}", config);

        Timer::after(Duration::from_millis(5000)).await; //added non blocking delay before checking again if wifi is connected 
    }
    
    esp_println!("Connected"); //boilerplate for checking connection

    spawner.spawn(read_current()).ok();

    Ok(()) //returns Result type if it's ok - no errors detected
}