use std::{
    io,
    process::{exit, Command, Output},
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use indoc::formatdoc;
use instekgpp::InstekGpp;
use serialport::SerialPortType;
use tracing::{error, info};

use crate::fail_test;

pub fn prepare_esp32(psu: &mut InstekGpp) {
    info!("Waiting for ESP32 JTAG/serial device...");

    let dev = match wait_for_esp32(Duration::from_secs(5)) {
        Ok(dev) => dev,
        Err(e) => {
            error!("Failed to find ESP32: {e}");
            fail_test(psu);
        }
    };

    info!("Found esp32 at {dev}");
    info!("Flashing target {dev} using esptool...");
    let output = match flash_esp32(&dev) {
        Ok(o) => o,
        Err(e) => {
            error!("Error using esptool: {e}");
            exit(-1);
        }
    };

    if !output.status.success() {
        error!(
            "Error flashing esp32:\n\n---stdout:---{}\n\n---stderr:---\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        exit(-1);
    }

    info!("Flashed esp32. Please press reset button.");
}

fn wait_for_esp32(time: Duration) -> Result<String> {
    let start = Instant::now();

    while start.elapsed() < time {
        for dev in serialport::available_ports()
            .map_err(|e| anyhow!("Error finding available serial ports: {e}"))?
        {
            let SerialPortType::UsbPort(port) = dev.port_type else {
                continue;
            };

            let Some(product) = port.product else {
                continue;
            };

            if product == "USB JTAG_serial debug unit" {
                return Ok(dev.port_name);
            }
        }

        sleep(Duration::from_millis(100));
    }

    Err(anyhow!("Timed out without finding ESP32."))
}

fn flash_esp32(port: &str) -> io::Result<Output> {
    Command::new("esptool.py")
        .args(
            formatdoc! {"
                --chip esp32s3
                --port {port}
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
        .output()
}
