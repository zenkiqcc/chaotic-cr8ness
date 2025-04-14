use feed_me_bits::{scan_devices, QrngDevice};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Quantum Leaks - QRNG Entropy Server");
    println!("Scanning for devices...");

    let devices = scan_devices()?;
    println!("\nFound {} QRNG device(s)", devices.len());

    for device in devices {
        println!("\nDevice Information:");
        println!("Vendor ID: 0x{:04x}", device.vendor_id());
        println!("Product ID: 0x{:04x}", device.product_id());
        println!("Manufacturer: {}", device.manufacturer()?);
        println!("Description: {}", device.description()?);
        println!("Serial: {}", device.serial()?);
    }

    // TODO: Implement API server
    println!("\nAPI server coming soon...");

    Ok(())
} 