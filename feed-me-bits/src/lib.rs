use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use std::error::Error;

// FTDI vendor ID
const FTDI_VID: u16 = 0x0403;

/// Represents a QRNG device
pub struct QrngDevice {
    device: Device<Context>,
    descriptor: DeviceDescriptor,
}

impl QrngDevice {
    /// Creates a new QrngDevice instance
    pub fn new(device: Device<Context>, descriptor: DeviceDescriptor) -> Self {
        Self { device, descriptor }
    }

    /// Returns the vendor ID of the device
    pub fn vendor_id(&self) -> u16 {
        self.descriptor.vendor_id()
    }

    /// Returns the product ID of the device
    pub fn product_id(&self) -> u16 {
        self.descriptor.product_id()
    }

    /// Returns the manufacturer string of the device
    pub fn manufacturer(&self) -> Result<String, Box<dyn Error>> {
        let handle = self.device.open()?;
        Ok(handle.read_manufacturer_string_ascii(&self.descriptor)?)
    }

    /// Returns the product description string of the device
    pub fn description(&self) -> Result<String, Box<dyn Error>> {
        let handle = self.device.open()?;
        Ok(handle.read_product_string_ascii(&self.descriptor)?)
    }

    /// Returns the serial number string of the device
    pub fn serial(&self) -> Result<String, Box<dyn Error>> {
        let handle = self.device.open()?;
        Ok(handle.read_serial_number_string_ascii(&self.descriptor)?)
    }
}

/// Scans for connected QRNG devices
pub fn scan_devices() -> Result<Vec<QrngDevice>, Box<dyn Error>> {
    let context = Context::new()?;
    let devices = context.devices()?;
    let mut qrng_devices = Vec::new();

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        // Check if this is an FTDI device
        if device_desc.vendor_id() == FTDI_VID {
            qrng_devices.push(QrngDevice::new(device.clone(), device_desc));
        }
    }

    Ok(qrng_devices)
} 