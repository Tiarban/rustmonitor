 
Final Year Project
 Status Report
“Rust-powered multi-channel energy monitor for smart home applications”
Name: Tiarnan Ryan
Student Number: 19309926

Project Description
The goal of this project is to develop an energy monitor using the Rust language which can monitor the energy usage for multiple devices via a single server. In order to implement this, there are many hurdles that need to be overcome - from learning the new syntax/logic of Rust to applying multithreaded networking to the embedded system. The scope of this project extends to embedded network programming with rust. Some of the challenges include choosing appropriate hardware and software elements to achieve this, such as an operating system. There are many areas to explore in regards to the programming of the device as there are many options for libraries. The implementation of the server also has many options - with the ability to choose between protocols such as WiFi, Bluetooth and Zigbee. There is also some choice available in the implementation of the server i.e whether to use HTTP server or direct socket communications via TCP/UDP packets.
Rust offers a memory-safe alternative to C++ with comparable speed. This language is ideal for embedded programming. I have found using the language to be easier to read than the alternatives due to the syntactic sugar of Rust which vastly improves the readability while adding minimal overhead. The lack of memory leaks also mitigates the effect of mistakes, making the programs safer. There's a wealth of open-source libraries available such as embedded-hal which are appropriate for the use case. 
	It is also essential to choose the OS that will be used in advance to the microcontroller to ensure compatibility. There are several open-source options available for this with various positives and negatives for different use cases. Ideally, the OS should meet the minimum requirements for the project (such as networking and asynchronous capabilities) while incurring minimum overhead from unneeded capabilities.
When it comes to embedded devices, it’s worth noting that the choice of the device should be capable of running the programs while also being cheap and power-efficient. Again, there is no shortage of choices but choosing an architecture that suits the ideal OS is preferable while being as cheap as possible.





Proposed Solution
Hardware/Software 
Arguably the most important choice in the project is that of the OS. This allows for the necessary abstractions to be made for networking and hardware. Specifically, the embedded-hal (hardware abstraction layer) and embedded-nal (network abstraction layer) packages should be integrated. It was also important to leverage open-source for the operating system so that all information is available. For these reasons I went with Embassy. Embassy also provides asynchronous functionality which limits the need for threading and multiple cores. [1]
The choices made in regards to the hardware used were dictated by the requirements of the project. Since there would be multiple client sensors, the microcontroller would ideally be cost-effective. The microcontroller also needed to be appropriate for IoT applications given that WiFi or Bluetooth would be necessary. For this reason, I went with the ESP family. I went with the ESP32C3 specifically since it was one of the only ESP microcontrollers supported by Embassy. The ESP32C3 is also built using the RISC-V ISA which I used heavily in my internship. The open-source nature of RISC-V is also thematic with Rust’s open-source philosophy. [2]
In order to learn the language Rust and how to use relevant containers for embedded systems, I used the Comprehensive Rust [3] program on github. The course covers everything from basic syntax to development of complex software using containers. The course is free and often used to introduce Rust to developers. 
For the analogue side, I decided to use signal generators in lieu of using actual appliances or high voltages and currents for safety reasons. This also eliminates the need for additional hardware to measure the signal and transmit it to the ESP reducing cost and potential failure points. The sensors will be connected to the signal generators.











Architecture
The design I chose for the architecture is a client-server model with an asynchronous connection handler which doesn’t block the server to allow for the transmission of data from multiple clients. A new handler task will be assigned to each client connecting to the server. The server will also be hosting the HTTP server which will be facilitating the transmission of data to the GUI from the server - leveraging Embassy’s asynchronous features[4]. This is necessary because the ESP32C3 is a single-core processor.



From left to right, the signals start in the signal generator which will be fed into the energy sensors. These sensors will be wired to input pins on the ESP32C3 devkit, which will then be relayed to the server via the clients. Each client is an ESP32C3 in STA mode. STA (Station) mode means that the device acts as an endpoint, only transmitting data to the central node. This is equivalent to a phone connecting to a WiFi router. The server ESP, however, will be in AP (Access Point) mode [5]. This means that it’s the central node of the network, facilitating connection between devices. Once the server has the data, it will be uploaded to the HTTP server, which is run concurrently on the same device. The data will be written on the server in JSON (JavaScript Object Notation) for easy parsing, as was done during my internship. The GUI will then be built with python on the laptop end and will also be responsible for any data analysis - offloading additional overhead from the microcontroller. The python GUI will use existing libraries such as PySimpleGUI. Matplotlib, a library I am familiar with from my internship, can be integrated into the GUI to display any graphs.



An alternative approach to the GUI is to write the html, css and javascript involved in displaying sensor information directly into the HTTP server. This might be easier to achieve than the custom python GUI by using some open source framework code. The drawback of this, however, is potentially over taxing the single core server. For this reason, the python implementation on the computer is the preferable approach.
Optionally, WPA-3 Personal encryption can be used. This significantly enhances the security of the network created by the ESPs. The ESP32C3 does support using this protocol, however it could incur additional overhead on both the server and client side and should be left until all necessary overhead is applied. [7]



In regards to the protocol for communicating with the sensors and the client, there are 3 choices: I2C, SPI and UART. Given that most sensors communicate with I2C, this is the protocol I'll be implementing.


The client ESPs will be configured to be masters, while each sensor will be in slave mode by default. This is because the clients will be initiating the transaction and setting the clock. An example of this protocol being implemented in rust can be found on youtube - but for a range-finder[10].
For the signal generator, there are no shortage of example devices with which the power can be monitored. The waves will likely be scaled down in order to be safely outputted by the signal generator.


Another potential option for this side of the project involves a CT (Current Transformer). The measured current will be induced in the second winding of the transformer - this means that rather than having the wire feed into the sensor, the sensor will simply wrap around the wire. This is safer, allowing the magnetic field around the wire to be measured. [12]




Plan


Week
Deliverables
Milestone
1
Template files written and git version control integration with VSCode - likely following setup from one of the linked github repositories. At this stage, most parts are already purchased and the toolchain is set up.
Current Milestone: 
Development environment set up, toolchain set up and github repository set up. Hardware needed for first milestone also already purchased.
2
Barebones client code with the goal of simply transmitting test data with serde or similar nal library. 
-Put in STA mode
-Template data
-Serialization


3
Barebones server code with goal of receiving test data for single client which is compatible with client code already written.
-Put in AP mode
-Sockets and ports for TCP or UDP.
-Temporary storing of data.


4
Barebones asynchronous connection handler to facilitate communication between 2 or more clients. At this stage, the client should be able to send test data from itself to the server ESP.
-Asynchronous adjustments
-Facilitate communication
-Purchase sensors
First Milestone:
Prototype client server communication
5
Implement I2C communication between a sensor and a client for real data transmission.
-Implement protocol on client
-Connect sensors to correct I2C pins


6
HTTP server implementation which stores collected by server at an endpoint asynchronously with data received from clients in JSON
-Write simple html
-Simple data endpoint at which data is available


7
Prototype python GUI which requests data from endpoint of HTTP server with ability to save information
-Should send get requests and display received information and status
Second Milestone:
Full prototype of system from sensor to GUI
8
Integrating python GUI with matplotlib to display graphs
-Finalise prototype GUI and implement matplotlib to display data received over time and energy usage pie chart


9
Include error control and test to find failure conditions - addressing each with a solution written.
-Disconnecting unexpectedly, timeouts etc.


10
Optional: Basic WPA-3 Personal implementation for both STA and AP mode ESPs.


11
Work on project report
Optional: improve GUI by adding more features and improve aesthetics
-Network information i.e. packet loss, latency
Third Milestone:
Fully operational and polished project
12
Finish full report




It should be noted that week 10 and 11 are optional to allow for unforeseen problems which may arise during development. Some parts of the project may end up taking slightly longer than anticipated and these two weeks can be used at any point during the project to stay on track. The optional points aren’t critical to the functioning of the project as a multi-channel energy monitor. The milestones are interpreted as progress by the end of the week, not the start.

4      References
[1] 	Embassy Homepage: 
https://embassy.dev/ 
Embassy on ESP32C3:
https://embassy.dev/book/#_getting_started

[2]	ESP32C3:
https://www.espressif.com/en/products/socs/esp32-c3

[3]	Comprehensive Rust: 
https://google.github.io/comprehensive-rust/index.html

[4]	Similar project using asynchronous HTTP server:
https://github.com/claudiomattera/esp32c3-embassy

[5]	ESP32C3 in STA and AP mode:
https://docs.espressif.com/projects/esp-idf/en/v4.4.3/esp32/api-reference/network/esp_wifi.html#:~:text=Station%20mode%20(aka%20STA%20mode,Stations%20connect%20to%20the%20ESP32.

[6]	PySimpleGUI image: 
	https://realpython.com/pysimplegui-python/

[7]	WPA-3 Personal encryption on ESP32C3:
https://docs.espressif.com/projects/esp-idf/en/v5.0/esp32/api-guides/wifi-security.html

[8]	WPA3 connection flowchart: 
https://www.researchgate.net/publication/344529445_Educational_modules_and_research_surveys_on_critical_cybersecurity_topics

[9]	I2C image and comparison information: 
https://www.totalphase.com/blog/2021/12/i2c-vs-spi-vs-uart-introduction-and-comparison-similarities-differences/?srsltid=AfmBOooH6PM2ebaqt-sejbUXtnR00saiOdJdSTYasfao6tHJVzfmMWik

[10]	I2C implementation example:
	https://www.youtube.com/watch?v=NOk1zimH75I

[11]	Typical signals: 
https://www.researchgate.net/figure/Typical-current-and-voltage-waveforms-of-nine-types-of-appliance_fig11_324082872

[12]	CT idea: 
https://simplyexplained.com/blog/Home-Energy-Monitor-ESP32-CT-Sensor-Emonlib/

[13]	CT information and diagram: 
	https://www.electronics-tutorials.ws/transformer/current-transformer.html
