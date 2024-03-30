use std::{
    io,
    process::{exit, Command, Output},
    thread::sleep,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use indoc::formatdoc;
use serialport::SerialPortType;
use tracing::{error, info};

use crate::EolTest;

impl EolTest {
    pub fn find_esp32(&mut self) -> String {
        info!("Waiting for ESP32 JTAG/serial device...");

        let dev = match self.wait_for_esp32(Duration::from_secs(5)) {
            Ok(dev) => dev,
            Err(e) => {
                error!("Failed to find ESP32: {e}");
                self.fail_test();
            }
        };

        info!("Found esp32 at {dev}");

        dev
    }

    pub fn prepare_esp32(&mut self) {
        info!("Waiting for ESP32 JTAG/serial device...");

        let dev = self.find_esp32();

        info!("Flashing target {dev} using esptool...");
        let output = match self.flash_esp32(&dev) {
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

        info!("Flashed esp32. Please press reset button if lights are off.");
    }

    fn wait_for_esp32(&self, time: Duration) -> Result<String> {
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

                #[cfg(target_os = "macos")]
                let normalized_dev_name = dev.port_name.replace("tty.usb", "cu.usb");
                #[cfg(target_os = "linux")]
                let normalized_dev_name = dev.port_name.clone();

                #[cfg(target_os = "macos")]
                let normalized_tester_name =
                    self.tester.name().unwrap().replace("tty.usb", "cu.usb");
                #[cfg(target_os = "linux")]
                let normalized_tester_name = self.tester.name().unwrap();

                if normalized_dev_name == normalized_tester_name {
                    continue; // skip the tester
                }

                #[cfg(target_os = "macos")]
                let expected_product_name = "USB JTAG_serial debug unit";

                #[cfg(target_os = "linux")]
                let expected_product_name = "USB_JTAG_serial_debug_unit";

                debug!("checking device with product name ${product}");

                if product == expected_product_name {
                    return Ok(dev.port_name);
                }
            }

            sleep(Duration::from_millis(100));
        }

        Err(anyhow!("Timed out without finding ESP32."))
    }

    fn flash_esp32(&self, port: &str) -> io::Result<Output> {
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
                ../build/fw/dut/bootloader.bin
                0x8000
                ../build/fw/dut/partitions.bin
                0xd000
                ../build/fw/dut/ota_data_initial.bin
                0x10000
                ../build/fw/dut/firmware.bin"
                }
                .split_ascii_whitespace(),
            )
            .output()
    }

    pub fn erase_flash(&mut self) -> io::Result<Output> {
        let dev = self.find_esp32();

        info!("Erasing flash using esptool...");
        Command::new("esptool.py")
            .args(
                formatdoc! {"
                --chip esp32s3
                --port {dev}
                erase_flash"
                }
                .split_ascii_whitespace(),
            )
            .output()
    }

    pub fn get_efuse_json(&mut self) -> io::Result<String> {
        let dev = self.find_esp32();

        info!("Reading efuses using espefuse...");
        Command::new("espefuse.py")
            .args(
                formatdoc! {"
                    --chip esp32s3
                    summary
                    --port {dev}
                    --format json"
                }
                .split_ascii_whitespace(),
            )
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .into_owned()
                    .lines()
                    .skip(4) // skip the beginning part of the output before the json starts
                    .collect()
            })
    }
}
