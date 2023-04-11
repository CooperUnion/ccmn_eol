use anyhow::{anyhow, Result};
use indoc::formatdoc;
use tracing::{info, error, Level, warn};
use serialport::SerialPortType;
use tracing_subscriber::FmtSubscriber;

use std::{
    process::{exit, Command},
    time::{self, Duration, Instant}, thread::sleep,
};

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("CCMN EOL Test ----");
    info!("Waiting for ESP32 JTAG/serial device...");

    let dev = wait_for_esp32(Duration::MAX).unwrap();

    info!("Found esp32 at {dev}");

    // run esptool to flash firmware
    info!("Flashing target {dev} using esptool...");
    let flash = Command::new("esptool.py")
        .args(
            formatdoc! {"
                --chip esp32s3
                --port {dev}
                --baud 460800 --before default_reset
                --after hard_reset write_flash
                -z
                --flash_mode dio
                --flash_freq 80m
                --flash_size 8MB 0x0
                ../build/fw/host/bootloader.bin
                0x8000
                ../build/fw/host/partitions.bin
                0xd000
                ../build/fw/host/ota_data_initial.bin
                0x10000
                ../build/fw/host/firmware.bin"
            }
            .split_ascii_whitespace(),
        )
        .output();

    let Ok(output) = flash else {
        error!("Error using esptool: {}", flash.unwrap_err());
        exit(-1);
    };

    if !output.status.success() {
        error!(
            "Error flashing esp32:\n  stdout:{}\n  stderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        exit(-1);
    }

    info!("Flashed esp32. Please press reset button.");

    // Tests:
    // 1. Power OK on 3v3, 5V
    // 2. ESP32 device present
    // 3. Flash testee firmware using script
    // 4. Check UART over serial/jtag using script
    // 5. Check UART over pins using tester
    // 6. Check EEPROM using self-test
    // 7. Check CAN using tester
    // 8. Check GPIO pins using tester
    // 9. Check PWM out using tester
    // 10. Check ADC in using tester
    // 11. Check power rails using script
    // 12. Erase flash
}

fn wait_for_esp32(time: Duration) -> Result<String> {
    let start = Instant::now();

    while start.elapsed() < time {
        for dev in serialport::available_ports().unwrap() {
            let SerialPortType::UsbPort(port) = dev.port_type else {
                continue;
            };

            let Some(product) = port.product else {
                continue;
            };

            if product == "USB JTAG_serial debug unit" {
                return Ok(dev.port_name)
            }
        }

        sleep(Duration::from_millis(100));
    }

    Err(anyhow!("No ESP32 found."))
}
