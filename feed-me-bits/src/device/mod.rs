use std::sync::Arc;
use rusb::{Context, Device, DeviceDescriptor, UsbContext};
use tokio::sync::Mutex;
use std::time::Duration;
use tracing::{info, warn, error};
use crate::error::QrngError;
use crate::{FTDI_VENDOR_ID, FTDI_PRODUCT_ID};
use std::collections::HashMap;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DeviceStatus {
    pub initialized: bool,
    pub temperature: f32,
    pub voltage: f32,
}

#[derive(Clone)]
pub struct DeviceManager {
    devices: Arc<Mutex<HashMap<String, QrngDevice>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_device(&self, device: QrngDevice) -> Result<String, QrngError> {
        let serial = device.serial().await?;
        let mut devices = self.devices.lock().await;
        devices.insert(serial.clone(), device);
        Ok(serial)
    }

    pub async fn remove_device(&self, serial: &str) -> Result<(), QrngError> {
        let mut devices = self.devices.lock().await;
        devices.remove(serial).ok_or_else(|| QrngError::DeviceNotFound(serial.to_string()))?;
        Ok(())
    }

    pub async fn get_device(&self, serial: &str) -> Result<QrngDevice, QrngError> {
        let devices = self.devices.lock().await;
        devices.get(serial)
            .cloned()
            .ok_or_else(|| QrngError::DeviceNotFound(serial.to_string()))
    }

    pub async fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().await;
        devices.keys().cloned().collect()
    }

    pub async fn initialize_device(&self, serial: &str) -> Result<(), QrngError> {
        let mut device = self.get_device(serial).await?;
        device.initialize().await?;
        self.add_device(device).await?;
        Ok(())
    }

    pub async fn read_entropy(&self, serial: &str, size: usize) -> Result<Vec<u8>, QrngError> {
        let device = self.get_device(serial).await?;
        device.read_entropy(size).await
    }

    pub async fn get_device_status(&self, serial: &str) -> Result<DeviceStatus, QrngError> {
        let device = self.get_device(serial).await?;
        device.status().await
    }
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