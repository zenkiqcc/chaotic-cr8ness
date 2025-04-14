pub mod error;
pub mod device;

pub use error::QrngError;
pub use device::{QrngDevice, DeviceStatus, scan_devices};

// FTDI vendor ID
const FTDI_VENDOR_ID: u16 = 0x0403;
// FTDI product ID for the QRNG device
const FTDI_PRODUCT_ID: u16 = 0x6001; 