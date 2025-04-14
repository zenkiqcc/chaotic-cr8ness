#[cfg(test)]
use super::*;
use tokio_test::block_on;
use tracing_subscriber::FmtSubscriber;
use std::sync::Arc;

#[tokio::test]
async fn test_device_initialization() {
    let mut devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    let mut device = devices.remove(0);
    let result = device.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_entropy_reading() {
    let mut devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    let mut device = devices.remove(0);
    device.initialize().await.unwrap();

    // Test reading a small amount of entropy
    let result = device.read_entropy(32).await;
    assert!(result.is_ok());
    let entropy = result.unwrap();
    assert_eq!(entropy.len(), 32);

    // Test reading a larger amount of entropy
    let result = device.read_entropy(1024).await;
    assert!(result.is_ok());
    let entropy = result.unwrap();
    assert_eq!(entropy.len(), 1024);
}

#[tokio::test]
async fn test_device_status() {
    let mut devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    let mut device = devices.remove(0);
    device.initialize().await.unwrap();

    let status = device.status().await;
    assert!(status.is_ok());
    let status = status.unwrap();
    
    // Verify status fields
    assert!(status.initialized);
    assert!(status.temperature > 0.0);
    assert!(status.voltage > 0.0);
}

#[tokio::test]
async fn test_concurrent_entropy_reading() {
    let mut devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    let mut device = devices.remove(0);
    device.initialize().await.unwrap();
    let device = Arc::new(device);

    // Spawn multiple tasks to read entropy concurrently
    let mut handles = vec![];
    for _ in 0..5 {
        let device = Arc::clone(&device);
        handles.push(tokio::spawn(async move {
            device.read_entropy(128).await
        }));
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        let entropy = result.unwrap();
        assert_eq!(entropy.len(), 128);
    }
}

#[tokio::test]
async fn test_entropy_quality() {
    let mut devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    let mut device = devices.remove(0);
    device.initialize().await.unwrap();

    // Read a large amount of entropy
    let entropy = device.read_entropy(1024 * 1024).await.unwrap();

    // Basic statistical tests
    let mut byte_counts = [0u32; 256];
    for &byte in &entropy {
        byte_counts[byte as usize] += 1;
    }

    // Check for uniform distribution
    let expected_count = entropy.len() / 256;
    let tolerance = (expected_count as f64 * 0.1) as u32; // 10% tolerance

    for &count in &byte_counts {
        assert!(
            (count as i32 - expected_count as i32).abs() <= tolerance as i32,
            "Byte distribution is not uniform enough"
        );
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mut devices = scan_devices().await.expect("Failed to scan devices");
    if devices.is_empty() {
        println!("No QRNG devices found. Skipping test.");
        return;
    }

    let mut device = devices.remove(0);

    // Test reading before initialization
    let result = device.read_entropy(32).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::DeviceNotInitialized));

    // Test invalid entropy size
    device.initialize().await.unwrap();
    let result = device.read_entropy(0).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), QrngError::InvalidState(_)));
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