use rusb::{Context, Device, DeviceDescriptor, DeviceHandle, UsbContext};
use std::time::Duration;

// FTDI vendor ID
const FTDI_VID: u16 = 0x0403;

fn main() {
    match scan_ftdi_devices() {
        Ok(_) => (),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn scan_ftdi_devices() -> rusb::Result<()> {
    let context = Context::new()?;
    
    println!("Scanning for FTDI devices...");
    let devices = context.devices()?;
    let mut ftdi_count = 0;

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        // Check if this is an FTDI device
        if device_desc.vendor_id() == FTDI_VID {
            ftdi_count += 1;
            print_device_info(&device, &device_desc)?;
        }
    }

    println!("\nFound {} FTDI device(s)", ftdi_count);
    Ok(())
}

fn print_device_info(device: &Device<Context>, device_desc: &DeviceDescriptor) -> rusb::Result<()> {
    let handle = device.open()?;
    
    println!("\nDevice Information:");
    println!("Vendor ID: 0x{:04x}", device_desc.vendor_id());
    println!("Product ID: 0x{:04x}", device_desc.product_id());
    
    // Get string descriptors
    if let Ok(manufacturer) = handle.read_manufacturer_string_ascii(&device_desc) {
        println!("Manufacturer: {}", manufacturer);
    }
    if let Ok(product) = handle.read_product_string_ascii(&device_desc) {
        println!("Description: {}", product);
    }
    if let Ok(serial) = handle.read_serial_number_string_ascii(&device_desc) {
        println!("Serial: {}", serial);
    }

    // Print USB version
    println!("USB Version: {}.{}", 
        device_desc.usb_version().major(),
        device_desc.usb_version().minor());

    Ok(())
} 