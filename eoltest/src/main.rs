use anyhow::{anyhow, Result};
use indoc::formatdoc;

use std::{
    process::{exit, Command},
    time::{self, Duration, Instant},
};

use cyme::{lsusb::profiler, system_profiler::USBDevice};

fn main() {
    println!("CCMN EOL Test ----");

    let dev = wait_for_esp32(Duration::MAX).unwrap();

    print!("{dev:#?}");

    print!("{}", dev.dev_path());

    exit(0);

    // run esptool to flash firmware
    eprintln!("flashing...");
    let flash = Command::new("esptool.py")
        .args(
            formatdoc! {"
                --chip esp32s3
                --port /dev/tty.usbmodem2101
                --baud 460800 --before default_reset
                --after hard_reset write_flash
                -z --flash_mode dio
                --flash_freq 80m
                --flash_size 8MB 0x0
                /Users/dmezh/ccmn_eol/build/fw/host/bootloader.bin
                0x8000
                /Users/dmezh/ccmn_eol/build/fw/host/partitions.bin
                0xd000
                /Users/dmezh/ccmn_eol/build/fw/host/ota_data_initial.bin
                0x10000
                /Users/dmezh/ccmn_eol/build/fw/host/firmware.bin"
            }
            .split_ascii_whitespace(),
        )
        .output();

    let Ok(output) = flash else {
        eprintln!("Error using esptool: {}", flash.unwrap_err());
        exit(-1);
    };

    if !output.status.success() {
        eprintln!(
            "Error flashing esp32:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
        exit(-2);
    }

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

fn wait_for_esp32(time: Duration) -> Result<USBDevice> {
    let start = Instant::now();

    while start.elapsed() < time {
        let spusb = profiler::get_spusb_with_extra(true).unwrap();

        for dev in spusb.flatten_devices() {
            if dev.name == "USB JTAG/serial debug unit" {
                return Ok(dev.clone());
            }
        }
    }

    Err(anyhow!("No ESP32 found."))
}
