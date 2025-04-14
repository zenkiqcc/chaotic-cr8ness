use std::sync::Arc;
use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use tokio::sync::Mutex;
use std::time::Duration;
use tracing::{info, warn, error};
use crate::error::QrngError;
use crate::{FTDI_VENDOR_ID, FTDI_PRODUCT_ID};

pub struct QrngDevice {
    device: Arc<Mutex<Device<Context>>>,
    descriptor: DeviceDescriptor,
    initialized: bool,
}

impl Clone for QrngDevice {
    fn clone(&self) -> Self {
        Self {
            device: Arc::clone(&self.device),
            descriptor: unsafe { std::ptr::read(&self.descriptor) },
            initialized: self.initialized,
        }
    }
}

pub struct DeviceStatus {
    pub initialized: bool,
    pub temperature: f32,
    pub voltage: f32,
}

impl QrngDevice {
    pub fn new(device: Device<Context>, descriptor: DeviceDescriptor) -> Self {
        Self {
            device: Arc::new(Mutex::new(device)),
            descriptor,
            initialized: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), QrngError> {
        let device = self.device.lock().await;
        let handle = device.open()?;
        
        // Reset device
        handle.reset()?;
        
        // Set configuration
        handle.set_active_configuration(1)?;
        
        // Claim interface
        handle.claim_interface(0)?;
        
        self.initialized = true;
        info!("QRNG device initialized successfully");
        Ok(())
    }

    pub async fn read_entropy(&self, size: usize) -> Result<Vec<u8>, QrngError> {
        if !self.initialized {
            return Err(QrngError::DeviceNotInitialized);
        }

        if size == 0 {
            return Err(QrngError::InvalidState("Invalid entropy size".to_string()));
        }

        let device = self.device.lock().await;
        let handle = device.open()?;
        let mut buffer = vec![0u8; size];
        let timeout = Duration::from_millis(1000);
        
        match handle.read_bulk(0x81, &mut buffer, timeout) {
            Ok(_) => {
                info!("Successfully read {} bytes of entropy", size);
                Ok(buffer)
            }
            Err(e) => {
                error!("Error reading entropy: {}", e);
                Err(QrngError::CommunicationError(e.to_string()))
            }
        }
    }

    pub async fn status(&self) -> Result<DeviceStatus, QrngError> {
        let device = self.device.lock().await;
        let handle = device.open()?;
        
        // Read status from device
        let mut buffer = [0u8; 2];
        let timeout = Duration::from_millis(100);
        
        match handle.read_bulk(0x82, &mut buffer, timeout) {
            Ok(_) => Ok(DeviceStatus {
                initialized: self.initialized,
                temperature: buffer[0] as f32,
                voltage: buffer[1] as f32 / 10.0,
            }),
            Err(e) => {
                warn!("Error reading device status: {}", e);
                Ok(DeviceStatus {
                    initialized: self.initialized,
                    temperature: 0.0,
                    voltage: 0.0,
                })
            }
        }
    }

    pub fn vendor_id(&self) -> u16 {
        self.descriptor.vendor_id()
    }

    pub fn product_id(&self) -> u16 {
        self.descriptor.product_id()
    }

    pub async fn manufacturer(&self) -> Result<String, QrngError> {
        let device = self.device.lock().await;
        let handle = device.open()?;
        Ok(handle.read_manufacturer_string_ascii(&self.descriptor)?)
    }

    pub async fn description(&self) -> Result<String, QrngError> {
        let device = self.device.lock().await;
        let handle = device.open()?;
        Ok(handle.read_product_string_ascii(&self.descriptor)?)
    }

    pub async fn serial(&self) -> Result<String, QrngError> {
        let device = self.device.lock().await;
        let handle = device.open()?;
        Ok(handle.read_serial_number_string_ascii(&self.descriptor)?)
    }
}

pub async fn scan_devices() -> Result<Vec<QrngDevice>, QrngError> {
    let context = Context::new()?;
    let devices = context.devices()?;
    let mut qrng_devices = Vec::new();

    for device in devices.iter() {
        let descriptor = device.device_descriptor()?;
        if descriptor.vendor_id() == FTDI_VENDOR_ID && descriptor.product_id() == FTDI_PRODUCT_ID {
            let qrng_device = QrngDevice::new(device, descriptor);
            info!("Found QRNG device: vendor={:04x}, product={:04x}", 
                qrng_device.vendor_id(), 
                qrng_device.product_id()
            );
            qrng_devices.push(qrng_device);
        }
    }

    info!("Found {} QRNG device(s)", qrng_devices.len());
    Ok(qrng_devices)
}

#[cfg(test)]
mod tests; 