# chaotic-cr8ness
Rust implementation of MeterFeeder (C++ FTDI MED USB QRNG driver) + entropy streaming/API server.

## Project Structure
This project is organized as a Cargo workspace containing two crates:

### Feed Me Bits (`feed-me-bits/`)
The core library crate that implements the FTDI MED USB QRNG driver functionality. It provides:
- Device detection and management
- Low-level USB communication
- Quantum entropy reading and processing
- Device status monitoring

### Quantum Leaks (`quantum-leaks/`)
The API server crate that serves quantum entropy to clients. It provides:
- Multiple API interfaces:
  - Endpoints for simple entropy requests
  - Realtime streaming for high-performance entropy delivery
- Device status monitoring
- Client authentication and rate limiting

## What is it?
A Rust implementation of MeterFeeder, split into a driver library and API server for serving quantum entropy from QRNG devices.

### The Naming
Chaotic Crateness. Origins in [MeterFeeder](https://github.com/vfp2/MeterFeeder).
* _Chaotic_ - alluding to the nature of quantum entropy/randomness
* _Cr8_ - alluding to Rust Crates
* _"Greatness"_ - in serving (feeding) said entropy to any consumers needing it

## Building
```bash
# Build all crates
cargo build

# Build and run the API server
cargo run -p quantum-leaks

# Build and run tests
cargo test
```

## License
Apache License 2.0 - see LICENSE file for details
