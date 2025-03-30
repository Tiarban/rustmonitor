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

use core;
use serde;

use embassy_net::{ //provides stack for tcp stuff etc. for communcation, not connection
    tcp::TcpSocket, IpEndpoint, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4
};
use esp_alloc as _;
use esp_hal::{clock::CpuClock, gpio::{self, Io, Pin}, i2c::{self, master::{Config, I2c}}, peripheral::{self, Peripheral}, peripherals::{self, Peripherals, I2C0, IO_MUX}, rng::Rng, timer::timg::TimerGroup, Blocking};
use esp_println::println;
use esp_wifi::{ //just for connection, embassy-net handles communication stuff in the stack
    init,
    wifi::{
        ClientConfiguration,
        Configuration,
        WifiDevice,
        WifiStaDevice,
    },
    EspWifiController,
};
use smoltcp::wire::Ipv4Address;
use serde_json_core;

use embedded_io_async::Write;
use ads1x1x::{channel::{self, SingleA0}, Ads1x1x, FullScaleRange, TargetAddr};

const PRECALCED_RECIP: f32 = 9.53674317e-7; //1 / (2^32 -1)*(4096) (u32 is random numb)


macro_rules! mk_static { //alternative to normal static cell for persistent storage (like nvs from esp-idf-svc) safer than static
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
#[derive(serde::Serialize)] //need to send this data so declaring serialisable
struct SensorData { //struct for storing sensor id and data (id should be based on ip)
    sensor_id: u8,
    sensor_value: f32,
}

#[embassy_executor::task]
async fn read_send_current (sta_stack: Stack<'static>, peripherals: I2C0, mut rng: Rng) { //for reading data

    fn generate_data(rng: &mut Rng) -> f32 {
        let mut x = rng.random() as f32; //gets random number
        x = PRECALCED_RECIP*x;
        return x;
    }
    
    let sda: gpio::GpioPin<8> = unsafe {gpio::GpioPin::steal()}; //bypass the ownership way to get the pins
    let scl: gpio::GpioPin<9> = unsafe {gpio::GpioPin::steal()};

    let my_i2c = I2c::new(peripherals, Config::default())
    .unwrap().with_sda(sda).with_scl(scl);
    let mut adc_i2c = Ads1x1x::new_ads1015(my_i2c, TargetAddr::Gnd); //declares the ads object with the i2c
    adc_i2c.set_data_rate(ads1x1x::DataRate12Bit::Sps250).unwrap(); //sets data rate at 250 samples per second (3.3kHz max)
    adc_i2c.set_full_scale_range(FullScaleRange::Within4_096V).unwrap(); //full scale range gets the voltage for +- 4096V


    let mut sta_rx_buffer  = [0; 1536];
    let mut sta_tx_buffer  = [0; 1536];



    let mut sta_socket = TcpSocket::new(sta_stack, &mut sta_rx_buffer, &mut sta_tx_buffer);
    sta_socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

   /* loop{
        esp_println::println!("Reading data: {:.1} From sensor: {}", data.sensor_value, data.sensor_id); //placeholder
        esp_println::println!("Sending data...");
        sta_socket.write_all(sendable.as_bytes()).await.ok(); //write all data to socket until end of buffer.len (not sure if ok() should be here)
        Timer::after(Duration::from_millis(1000)).await; //return ctrl to executor
    } */

    let server_ip = Ipv4Address::new(192, 168, 2, 1);
    let server_port = 5050; //tcp port on server
    let server_endpoint = IpEndpoint::new(server_ip.into(), server_port); //creates endpoint for connection

    loop { //connect to socket
        match sta_socket.connect(server_endpoint).await {
            Ok(_) => {
                println!("Connected to server at port {}", server_endpoint.port);
                break;
            }
            Err(e) => {
                println!("Connection failed: {:?}", e);
                Timer::after(Duration::from_secs(1)).await; // retry after delay
            }
        }
    }

    loop { //send data loop, also moved data declaration here so it can be updated within the loop - static maybe?
        let raw_data = nb::block!(adc_i2c.read(SingleA0)).unwrap(); //reads on channel a0 on adc
        let voltage = (raw_data as f32/2048.0)*4096.0;
        println!("{}", voltage);
        //let fake_voltage = generate_data(&mut rng); //replace with real voltage above for actual reading
        //println!("{}", fake_voltage);
        let data = SensorData { sensor_id: 20, sensor_value: voltage}; //placeholder for reading current via i2c
        //let jbuffer = [0u8; 1024]; //128 birs fine probably since struct is small note: couldnt get buffer to work so string instead
        let mut sendable:String<128>  = serde_json_core::to_string(&data).unwrap(); //slice up data for transmission
        sendable.push_str("\r\n\r\n").unwrap(); //append so i can identify end of string at server
        sta_socket.write_all(sendable.as_bytes()).await.ok(); //write strings to socket (.ok() is questionable but compiler wants it)
    }
}


#[embassy_executor::task]
async fn sta_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await //runs network stack
}

/* 
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
                    //auth method could be used later for WPA3 implementation
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
    
} */

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {

    esp_println::logger::init_logger_from_env();

    esp_println::println!("Init!");

    //handles:
    //wifi stuff

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max()); //sets clock to max clock speed
    let peripherals = esp_hal::init(config);//get peripheral using esp_hal abstraction

    esp_alloc::heap_allocator!(72 * 1024); //have to manually set heap

    let timg0 = TimerGroup::new(peripherals.TIMG0); //first timer for peripheral stuff

    let timg1 = TimerGroup::new(peripherals.TIMG1); //embassy timer for async stuff
        esp_hal_embassy::init(timg1.timer0);

    let mut rng = Rng::new(peripherals.RNG); // random number generation for dhcp and initialisation of controller
    
    //i2c stuff
    //let io = Io::new(peripherals.IO_MUX); i think this is the safe way but cant get it to work
    let per_pins = peripherals.I2C0;

        //and Gnd as the adress -> since we only have one channel its fine


    //wifi stuff
    let init = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap() //switched rng.clone to into, not sure the consequences
    );

    let wifi = peripherals.WIFI; //handle for peripherals

    let (_wifi_ap_interface, wifi_sta_interface, mut wifi) =
        esp_wifi::wifi::new_ap_sta(&init, wifi).unwrap(); //_ap_interface wont be used, just the sta_interface. Controller is now how i do everything like connecting etc similar to how 
        //it was done in esp-idf-svc. 

     /*pub fn new<M: WifiModemPeripheral>(
        modem: impl Peripheral<P = M> + 'd,
        sysloop: EspSystemEventLoop,
        nvs: Option<EspDefaultNvsPartition>
    ) -> Result<Self, EspError> */ // this is the signature of the method to get wifi driver handle

    let mut ssid = String::<32>::new();
    ssid.push_str("esp-wifi").unwrap(); //explicit type required for config for both Three_E42796

    let mut password = String::<64>::new();
    password.push_str("").unwrap(); //2hG{w?24

    let mut dns_servers = Vec::<Ipv4Address, 3>::new(); //setting DNS vector, this is googles for default
    dns_servers.push(Ipv4Address::new(8, 8, 8, 8)).unwrap();

    let sta_config = embassy_net::Config::ipv4_static(StaticConfigV4 {  //setting static ipv4 address since i couldnt get dhcp working in this context and i think itll be easier to identify the sensors
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 20), 24), // ip for client
        gateway: Some(Ipv4Address::new(192, 168, 2, 1)), // ip for server router
        dns_servers, 
    });

    wifi.set_configuration(&Configuration::Client(ClientConfiguration { //configuration for client
        ssid,
        auth_method: esp_wifi::wifi::AuthMethod::None,
         //explicit defining done above - examples for using .into() or from here dont work
                    //auth method could be used later for WPA3 implementation
        ..Default::default() //fills non specified methods with default config
    })).unwrap(); //note these brackets are closed here for the nested struture of accessing the clientconfig 
    
    wifi.start().unwrap(); //start_async made in esp-wifi for async ops
    
    //wifi.connect_async().await.unwrap(); //self explanatory lines
    
    while !wifi.is_connected().unwrap() {
        // Get and print connetion configuration
        wifi.connect().unwrap(); //self explanatory lines
        let config = wifi.configuration().unwrap();
        esp_println::println!("Waiting for station {:?}", config);
    
        Timer::after(Duration::from_millis(5000)).await; //added non blocking delay before checking again if wifi is connected 
    }
    
    esp_println::println!("Connected" ); //boilerplate for checking connection

    let seed = (rng.random() as u64) << 32 | rng.random() as u64; //set rng seed


    let (sta_stack, sta_runner) = embassy_net::new( //getting the network stack 
        wifi_sta_interface,
        sta_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(sta_task(sta_runner)).ok();


 loop {
    if let Some(ip_config) = sta_stack.config_v4() {
        esp_println::println!("IP Address: {:?}", ip_config.address);
        break;
    } else {
        esp_println::println!("Since DHCP is disabled here, IP address failed to be assigned");
    }
    Timer::after(Duration::from_secs(1)).await;
}

    let generate_data_rng = rng.clone(); //clone the rng so that theres no ownership issues w/ wifi controller

    spawner.spawn(read_send_current(sta_stack, per_pins, generate_data_rng)).ok();

loop {
    Timer::after(Duration::from_secs(5)).await;
}
}