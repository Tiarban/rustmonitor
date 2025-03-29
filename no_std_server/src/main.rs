//! Embassy access point
//!
//! - creates an open access-point with SSID `esp-wifi`
//! - you can connect to it using a static IP in range 192.168.2.2 .. 192.168.2.255, gateway 192.168.2.1
//! - open http://192.168.2.1:8080/ in your browser - the example will perform an HTTP get request to some "random" server
//!
//! On Android you might need to choose _Keep Accesspoint_ when it tells you the WiFi has no internet connection, Chrome might not want to load the URL - you can use a shell and try `curl` and `ping`
//!
//! Because of the huge task-arena size configured this won't work on ESP32-S2
//!

//% FEATURES: embassy esp-wifi esp-wifi/wifi esp-wifi/utils esp-wifi/sniffer esp-hal/unstable
//% CHIPS: esp32 esp32s2 esp32s3 esp32c2 esp32c3 esp32c6

#![no_std]
#![no_main]

use core::{default, fmt, net::Ipv4Addr, str::FromStr, sync::atomic::{AtomicU32, Ordering}};

use embassy_executor::Spawner;
use embassy_net::{
    tcp::TcpSocket, IpEndpoint, IpListenEndpoint, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4
};
use embassy_time::{Duration, Timer, Instant};
//use embassy_sync::lazy_lock;
use embedded_io_async::Write;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, rng::Rng, timer::timg::TimerGroup};
use esp_println::{print, println};
use esp_wifi::{
    init,
    wifi::{
        AccessPointConfiguration,
        Configuration,
        WifiApDevice,
        WifiController,
        WifiDevice,
        WifiEvent,
        WifiState,
    },
    EspWifiController,
};
use heapless::String;
use core;
use serde;
use serde_json_core;
use core::fmt::Write as writer; //write already defined in embeddedioasync
use circular_buffer::CircularBuffer;



//need to send this data so declaring serialisable
#[derive(serde::Deserialize)] //makes the sensordata deserialisable for processing in handler
struct SensorData { //struct for storing sensor id and data (id should be based on ip)
    sensor_id: u8,
    sensor_value: f32,
}
//using embassy time to get the time of the reading relative to boot from Instant
#[derive(Copy, Clone, serde::Serialize)] //copy and clone needed for default values to be copied for affys 
struct TimedSensorData {
    sensor_id: u8,
    sensor_value: f32,
    sensor_time: u64,
}

#[derive(serde::Serialize, Clone, Copy)]
struct ClientReadings {
    readings: [TimedSensorData; 8],
}

#[derive(serde::Serialize)]
struct TotalClientReadings { //this allows me to seriailse all into one json object
    client1: ClientReadings,
    client2: ClientReadings,
    client3: ClientReadings,
    client4: ClientReadings,
}

impl fmt::Display for SensorData {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{} : {}", self.sensor_id, self.sensor_value)
    }
}

impl Default for TimedSensorData { //to generate default values
    fn default() -> Self {
        TimedSensorData {
            sensor_id: 0,
            sensor_value: 0.0,
            sensor_time: 0,
        }
    }
}

impl Default for ClientReadings { //generates default values using timedsensordata default
    fn default() -> Self {
        ClientReadings {
            readings: [TimedSensorData::default(); 8]
        }
    }
}
//following two consts declared manually rather than using default
const TIMED_SENSOR_DATA_DEFAULT: TimedSensorData = TimedSensorData{
    sensor_id: 0,
    sensor_time: 0,
    sensor_value: 0.0,
};

const CLIENT_READINGS_DEFAULT: ClientReadings = ClientReadings{
    readings: [TIMED_SENSOR_DATA_DEFAULT; 8]
};


static MAX_CLIENTS: usize = 4; //maximum number of clients allowed - limits buffer pool

static CLIENT_COUNT: AtomicU32 = AtomicU32::new(0); //keeps track of client number using static atomic variable
static mut RX_CLIENT_BUFFER_POOL: [[u8; 1536]; MAX_CLIENTS] = [[0u8; 1536]; MAX_CLIENTS]; //using static pool instead of the mut array for buffers to get around lifetime issues when spawning clients *note u8 is the size of each element
static mut TX_CLIENT_BUFFER_POOL: [[u8; 1536]; MAX_CLIENTS] = [[0u8; 1536]; MAX_CLIENTS]; //for client tx
static mut RX_GUI_BUFFER: [u8; 1536] = [0u8;1536]; //for http
static mut TX_GUI_BUFFER: [u8; 1536] = [0u8;1536];

//declaring seperate static fields to store current data

static mut DATA_10: ClientReadings = CLIENT_READINGS_DEFAULT;
static mut DATA_15: ClientReadings = CLIENT_READINGS_DEFAULT;
static mut DATA_20: ClientReadings = CLIENT_READINGS_DEFAULT;
static mut DATA_25: ClientReadings = CLIENT_READINGS_DEFAULT;

static mut BUF_10: CircularBuffer<8, TimedSensorData> = CircularBuffer::<8, TimedSensorData>::new();
static mut BUF_15: CircularBuffer<8, TimedSensorData> = CircularBuffer::<8, TimedSensorData>::new();
static mut BUF_20: CircularBuffer<8, TimedSensorData> = CircularBuffer::<8, TimedSensorData>::new();
static mut BUF_25: CircularBuffer<8, TimedSensorData> = CircularBuffer::<8, TimedSensorData>::new();


// When you are okay with using a nightly compiler it's better to use https://docs.rs/static_cell/2.1.0/static_cell/macro.make_static.html
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const GW_IP_ADDR_ENV: Option<&'static str> = option_env!("GATEWAY_IP");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = Rng::new(peripherals.RNG);

    let init = &*mk_static!(
        EspWifiController<'static>,
        init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
    );

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, wifi, WifiApDevice).unwrap();

    
    let timg1 = TimerGroup::new(peripherals.TIMG1); //removed conditional compilation as its unnecessary for my use-case
    esp_hal_embassy::init(timg1.timer0);
        

    let gw_ip_addr_str = GW_IP_ADDR_ENV.unwrap_or("192.168.2.1");
    let gw_ip_addr = Ipv4Addr::from_str(gw_ip_addr_str).expect("failed to parse gateway ip");

    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(gw_ip_addr, 24),
        gateway: Some(gw_ip_addr),
        dns_servers: Default::default(),
    });

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<7>, StackResources::<7>::new()),
        seed,
    );

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();
    spawner.spawn(run_dhcp(stack, gw_ip_addr_str)).ok();

    let mut _rx_buffer = [0; 1536];
    let mut _tx_buffer = [0; 1536];


    loop {
        if stack.is_link_up() { //checks if link is down, if down wait 500ms
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    println!(
        "Connect to the AP `esp-wifi` and point your browser to http://{gw_ip_addr_str}:8080/"
    );
    println!("DHCP is enabled so there's no need to configure a static IP, just in case:");
    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }
    stack
        .config_v4()
        .inspect(|c| println!("ipv4 config: {c:?}"));

unsafe{  //static mutable variables arent safe but fine for now probably
    let mut socket = TcpSocket::new(stack, &mut RX_GUI_BUFFER, &mut TX_GUI_BUFFER); //socket should handle http connections
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    spawner.spawn(gui_handler(socket)).ok();

    /* 
    let mut client_rx_buffer = [0; 1536];
    let mut client_tx_buffer = [0; 1536];
    
    let mut client_socket = TcpSocket::new(stack, &mut client_rx_buffer, &mut client_tx_buffer); //socket should handle client connections
    client_socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));
*/

    loop {

        println!("Building new tcp socket for client...");
        let new_client_id: usize  = CLIENT_COUNT.load(Ordering::Relaxed).try_into().unwrap(); //load client count to represent the address of static buffer
        println!("Current client count: {}", new_client_id);
        let mut client_socket = TcpSocket::new(stack, &mut RX_CLIENT_BUFFER_POOL[new_client_id], &mut TX_CLIENT_BUFFER_POOL[new_client_id]);
        client_socket.set_timeout(Some(embassy_time::Duration::from_secs(60)));

        //following moved from handler to here so after socket is made, increment is done immediatley
        let current_count = CLIENT_COUNT.load(Ordering::Relaxed); //reload client count
        CLIENT_COUNT.store(current_count.wrapping_add(1), Ordering::Relaxed); //modify and store with relaxed ordering for now
        

        println!("Waiting for client connection..."); //accept requests from client
        let r = client_socket
            .accept(IpListenEndpoint {
                addr: None, //across all ip addresses
                port: 5050,
            })
            .await;
        match r {
            Ok(()) =>{
                println!("Client handler spawned: {:?}", client_socket.remote_endpoint());
                spawner.spawn(client_handler(client_socket)).ok();
        },
            Err(e) => println!("Client connection error: {:?}", e),
        }


        if let Err(e) = r {
            print!("Client connection error: {:?}", e);
        }
        /*match client_socket.accept(IpListenEndpoint { //pattern matches the accepted return for error and for ok
            addr: None,
            port: 8080,
        }).await {
            Ok(mut comm_socket) => {
                spawner.spawn(client_handler(comm_socket));
            }
            Err(e) => {println!("Failed to connect: {:?}", e)}
        }

        if let Err(e) = r {
            println!("Server connection error: {:?}", e);
            continue;
        }
            */

    }
    }
}

#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>, gw_ip_addr: &'static str) {
    use core::net::{Ipv4Addr, SocketAddrV4};

    use edge_dhcp::{
        io::{self, DEFAULT_SERVER_PORT},
        server::{Server, ServerOptions},
    };
    use edge_nal::UdpBind;
    use edge_nal_embassy::{Udp, UdpBuffers};

    let ip = Ipv4Addr::from_str(gw_ip_addr).expect("dhcp task failed to parse gw ip");

    let mut buf = [0u8; 1500];

    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];

    let buffers = UdpBuffers::<3, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();

    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(ip),
            &ServerOptions::new(ip, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|e| log::warn!("DHCP server error: {e:?}"));
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn gui_handler(mut socket: TcpSocket<'static>) { //super unsure about this stuff, but it should only connect once and continously read, not exiting
loop { //first loop checks connection, inner loop reads until done.
        println!("Waiting for GUI connection..."); //accept http requests from GUI
        let r = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 8080,
            })
            .await;
        println!("Connected to GUI...");

        if let Err(e) = r {
            println!("GUI connection error: {:?}", e);
        }

        //use embedded_io_async::Write;

        let mut buffer = [0u8; 1024];
        let mut pos = 0;
        loop {
            match socket.read(&mut buffer).await {
                Ok(0) => {
                    println!("read EOF");
                    break; //used to be break, but i dont want to shut socket down at EOF
                }
                Ok(len) => {
                    let to_print =
                        unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };
    
                    if to_print.contains("\r\n\r\n") {
                        print!("{}", to_print);
                        println!();
                        break; 
                    }
                    pos += len;
                }
                Err(e) => {
                    println!("read error: {:?}", e);
                    break;
                }
            }
        }
        unsafe {
            /*
            circ_to_readings(&BUF_10, &mut DATA_10);
            circ_to_readings(&BUF_15, &mut DATA_15);
            circ_to_readings(&BUF_20, &mut DATA_20);
            circ_to_readings(&BUF_25, &mut DATA_25);
            */
            let totalreadings: TotalClientReadings = TotalClientReadings {
                client1: DATA_10,
                client2: DATA_15,
                client3: DATA_20,
                client4: DATA_25,
            };
            let jsonpayload:String<2000>  = serde_json_core::to_string(&totalreadings).unwrap();
            let mut webpage: String<3000> = String::new(); //need to use strings constructor, not different string val
            write!( webpage,
                "HTTP/1.0 200 OK\r\nContent-Type: application/json\r\n\r\n{}",
            jsonpayload,
            //put data here
            ).unwrap();
            //following code handles posting to http socket. unnecessary for now but needed for transmitting to GUI");
            let r = socket 
                .write_all(
                    webpage.as_bytes()) //displays method on browser
                .await;
            if let Err(e) = r {
                println!("write error: {:?}", e);
            }
        }
        let r = socket.flush().await;
                    if let Err(e) = r {
            println!("flush error: {:?}", e);
        }
        Timer::after(Duration::from_millis(1000)).await;

        socket.close();
        Timer::after(Duration::from_millis(1000)).await;

        socket.abort();
    
}
}

#[embassy_executor::task(pool_size = MAX_CLIENTS)] //ties the max instances to client size from pool
async fn client_handler(mut client_socket: TcpSocket<'static>) { //this task will read from the socket
    let mut buffer = [0u8; 1024]; //same buffer size as transmitted data
    let mut pos = 0;
    let mut handle = TimedSensorData {sensor_id: 0, sensor_value: 0.0, sensor_time:0}; //temp for storage of recieved values
    fn circ_to_readings(buf: &CircularBuffer<8, TimedSensorData>, output: &mut ClientReadings) {//&mut since it has to alter the real output
        let mut i = 0;
        for elem in buf.iter() {
            output.readings[i] = *elem;
            i+=1;
        }
    }
    
    loop{
        match client_socket.read(&mut buffer).await { //match against buffer contents
            Ok(0) => {
                println!("read EOF"); //client is no longer streaming
                break; //maybe should be break, continue for now
            }
            Ok(len) => {
                let to_print =
                    unsafe { core::str::from_utf8_unchecked(&buffer[..(pos + len)]) };

                if to_print.contains("\r\n\r\n") {
                    let parts = to_print.split("\r\n\r\n"); //splits based on delimiter
                    for part in parts {
                        let temp = part.trim(); //trimes whitespace \r\n\r\n
                        match serde_json_core::from_str::<SensorData>(temp){
                            Ok((recieved,_)) => {
                                println!("Data succesfuly parsed: {}", recieved);
                                handle.sensor_id = recieved.sensor_id;
                                handle.sensor_value = recieved.sensor_value;
                                handle.sensor_time = Instant::now().as_millis();
                                print!("{}", handle.sensor_time)
                            } //becuase it expects (Sensordata, usize)
                            Err(e) => {println!("Parsing Error: {:?}", e)}
                        }
                    }
                    //print!("{}", to_print); //hopefully unnecessary...
                    unsafe{ //messing with static muts requires unsafe code
                        match client_socket.remote_endpoint() {
                            Some(IpEndpoint {port: _, addr}) if addr == embassy_net::IpAddress::Ipv4(Ipv4Addr::new(192, 168, 2, 10)) => {
                                BUF_10.push_back(handle);
                                circ_to_readings(&BUF_10, &mut DATA_10);
                            }
                            Some(IpEndpoint {port: _, addr}) if addr == embassy_net::IpAddress::Ipv4(Ipv4Addr::new(192, 168, 2, 15)) => {
                                BUF_15.push_back(handle);
                                circ_to_readings(&BUF_15, &mut DATA_15);
                            }
                            Some(IpEndpoint {port: _, addr}) if addr == embassy_net::IpAddress::Ipv4(Ipv4Addr::new(192, 168, 2, 20)) => {
                                BUF_20.push_back(handle);
                                circ_to_readings(&BUF_20, &mut DATA_20);
                            }
                            Some(IpEndpoint {port: _, addr}) if addr == embassy_net::IpAddress::Ipv4(Ipv4Addr::new(192, 168, 2, 25)) => {
                                BUF_25.push_back(handle);
                                circ_to_readings(&BUF_25, &mut DATA_25);

                            }
                            _=> {}
                        }
                    }
                    pos = 0; //reset position at delimiter detection
                    continue; //used to be break, but i dont want to shut socket down when receiving delimiter expression, continue reading
            }
            pos += len; //increment position by the current size if no end detected
        }
        Err(e) => {
            println!("read error: {:?}", e);
            break; //break only on error
        }
        }
    }

    let r = client_socket.flush().await; //this block gracefully flushes and shuts down socket when break happens
    if let Err(e) = r {
        println!("flush error: {:?}", e);
    }
    Timer::after(Duration::from_millis(1000)).await;

    let current_count_post = CLIENT_COUNT.load(Ordering::Relaxed);
    CLIENT_COUNT.store(current_count_post.wrapping_sub(1), Ordering::Relaxed); //frees up a buffer used when created 

    client_socket.close();
    Timer::after(Duration::from_millis(1000)).await;

    client_socket.abort();
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::ApStarted => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::AccessPoint(AccessPointConfiguration { //configures access point w/SSID esp-wifi and no password
                ssid: "esp-wifi".try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static, WifiApDevice>>) {
    runner.run().await //runs network stack
}