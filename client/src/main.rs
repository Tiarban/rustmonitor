use std::{thread::sleep, time::Duration, sync::{Mutex, Arc}, str::from_utf8, collections::HashMap};

use anyhow::{self, Error};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;

const SSID: &str = "";
const PASS: &str = ""; //should be hardcoded for now

fn main() {
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

    let mut wifi = EspWifi::new(peripherals.modem, sysloop, Some(nvs))?; // i/p to 'new' method on line 26
    //wifi mut is now the wifi handle driver (like peripherals)

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
        ssid: "temp ssid".into(),
        password: "temp pass".into(),
        auth_method: AuthMethod::None, //could be used later for WPA3 implementation
        ..Default::default() //fills non specified methods with default config
    }))?; //note these brackets are closed here for the nested struture of accessing the clientconfig 

    wifi.start()?;

    wifi.connect()?; //self explanatory lines

    while !wifi.is_connected().unwrap() {
        // Get and print connetion configuration
        let config = wifi.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
    }
    
    println!("Connected"); //boilerplate for checking connection

    Ok(()) //returns Result type if it's ok - no errors detected
}