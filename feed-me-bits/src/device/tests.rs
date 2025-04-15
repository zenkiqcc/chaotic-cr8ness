#[cfg(test)]
use super::*;
use tokio_test::block_on;
use tracing_subscriber::FmtSubscriber;

#[tokio::test]
async fn test_device_manager() {
    let manager = DeviceManager::new();
    let devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    // Add all devices to the manager
    let mut serials = Vec::new();
    for device in devices {
        let serial = manager.add_device(device).await.expect("Failed to add device");
        serials.push(serial);
    }

    // List devices
    let device_list = manager.list_devices().await;
    assert_eq!(device_list.len(), serials.len());
    for serial in &serials {
        assert!(device_list.contains(serial));
    }

    // Initialize each device
    for serial in &serials {
        manager.initialize_device(serial).await.expect("Failed to initialize device");
    }

    // Test reading entropy from each device
    for serial in &serials {
        let entropy = manager.read_entropy(serial, 32).await.expect("Failed to read entropy");
        assert_eq!(entropy.len(), 32);
    }

    // Test getting status from each device
    for serial in &serials {
        let status = manager.get_device_status(serial).await.expect("Failed to get device status");
        assert!(status.initialized);
    }

    // Test concurrent entropy reading from all devices
    let mut handles = vec![];
    for serial in &serials {
        let manager = manager.clone();
        let serial = serial.clone();
        handles.push(tokio::spawn(async move {
            let entropy = manager.read_entropy(&serial, 128).await.expect("Failed to read entropy");
            assert_eq!(entropy.len(), 128);
        }));
    }

    // Wait for all concurrent reads to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Test removing devices
    for serial in &serials {
        manager.remove_device(serial).await.expect("Failed to remove device");
    }

    // Verify devices are removed
    let device_list = manager.list_devices().await;
    assert!(device_list.is_empty());
}

#[tokio::test]
async fn test_device_manager_error_handling() {
    let manager = DeviceManager::new();

    // Test getting non-existent device
    let result = manager.get_device("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::DeviceNotFound(_)));

    // Test removing non-existent device
    let result = manager.remove_device("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::DeviceNotFound(_)));

    // Test initializing non-existent device
    let result = manager.initialize_device("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::DeviceNotFound(_)));

    // Test reading entropy from non-existent device
    let result = manager.read_entropy("non-existent", 32).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::DeviceNotFound(_)));

    // Test getting status from non-existent device
    let result = manager.get_device_status("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::DeviceNotFound(_)));
}

#[test]
fn test_scan_devices() {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_ansi(false)
        .init();

    // Scan for devices
    let result = block_on(scan_devices());
    assert!(result.is_ok(), "Failed to scan devices: {:?}", result.err());

    let devices = result.unwrap();
    println!("\nFound {} QRNG device(s)", devices.len());

    // Print details for each device
    for (i, device) in devices.iter().enumerate() {
        println!("\nDevice {}:", i + 1);
        println!("  Vendor ID: 0x{:04x}", device.vendor_id());
        println!("  Product ID: 0x{:04x}", device.product_id());

        if let Ok(manufacturer) = block_on(device.manufacturer()) {
            println!("  Manufacturer: {}", manufacturer);
        }

        if let Ok(description) = block_on(device.description()) {
            println!("  Description: {}", description);
        }

        if let Ok(serial) = block_on(device.serial()) {
            println!("  Serial: {}", serial);
        }

        if let Ok(status) = block_on(device.status()) {
            println!("  Status:");
            println!("    Initialized: {}", status.initialized);
            println!("    Temperature: {:.1}°C", status.temperature);
            println!("    Voltage: {:.1}V", status.voltage);
        }
    }
} 