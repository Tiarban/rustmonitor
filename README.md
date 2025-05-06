# Rust-Powered Multi-Channel Energy Monitor

A bare-metal Rust client–server system for real-time, multi-channel home-energy monitoring. ESP32-C3 devices sample voltage via ADS1015 ADCs, send JSON over TCP to a Rust-powered server, and display data in a Python/Qt GUI.

## Features

- **Multi-channel sampling** on ESP32-C3 + ADS1015 (12-bit, 3.3 kHz)  
- **Asynchronous Rust** (no_std + Embassy) for client & server  
- **Reliable TCP transport** with JSON serialization (serde_json_core)  
- **Desktop GUI** in Python/PySide + QML + PyQtGraph  
- **Lightweight binaries** (~180 KB) and sub-100 ms round-trip latency  

## Hardware Requirements

- 1 × ESP32-C3 dev kit (RISC-V, 400 KB SRAM, Wi-Fi) per channel  
- 1 × Adafruit ADS1015 I²C ADC per ESP32  
- Breadboard, pull-ups (10 kΩ), wiring cables  
- USB power for ESP32 and ADALM2000 (for test signals)  

## Software Requirements

- Rust (nightly) + Cargo  
- `espflash` for flashing:  
  ```bash
  cargo install espflash 


## License

This project is licensed under the [Apache License 2.0](LICENSE).

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
