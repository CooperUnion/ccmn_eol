use clap::Parser;
use serde::Serialize;
use serialport::SerialPort;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use std::{
    fs::{self},
    process::exit,
    thread::sleep,
    time::Duration,
};

mod esp32;
mod tester;

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "macos"))] {
        use instekgpp::InstekGpp;
        mod power;
    }
}

#[derive(clap::Parser)]
struct Args {
    #[clap(long)]
    tester_port: String,
    #[clap(long)]
    serial_number: String,
}

struct EolTest {
    #[cfg(not(target_os = "macos"))]
    psu: InstekGpp,
    tester: Box<dyn SerialPort>,
}

impl EolTest {
    fn main() -> ! {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let args = Args::parse();

        info!("CCMN EOL Test ----");

        // try to open tester port
        let tester = serialport::new(args.tester_port, 115200).open().unwrap();

        #[cfg(not(target_os = "macos"))]
        let psu = power::prepare_psu();

        #[cfg(not(target_os = "macos"))]
        let mut eol = EolTest { psu, tester };
        #[cfg(target_os = "macos")]
        let mut eol = EolTest { tester };

        eol.prepare_esp32();

        info!("Waiting for 5 seconds...");
        sleep(Duration::from_secs(5));

        let results = loop {
            match eol.get_test_result() {
                Ok(res) => break res,
                Err(e) => {
                    warn!("Error getting test results: {e} Trying again.");
                    continue;
                }
            }
        };

        info!("Got test results.");

        match results.gpio_result {
            true => info!("GPIO test PASS."),
            false => error!("!!! GPIO test FAIL!"),
        };

        match results.adc_result {
            Some((_pin, tol)) => info!("ADC test PASS. Largest tolerance was {} mV", tol.abs()),
            None => error!("!!! ADC test FAIL!"),
        };

        match results.eeprom_result {
            0 => error!("!!! EEPROM test NOT RUN!"),
            1 => info!("EEPROM test PASS."),
            2 => error!("!!! EEPROM test FAIL"),
            _ => panic!("Unexpected value for eeprom_result."),
        };

        if results.gpio_result && results.adc_result.is_some() && results.eeprom_result == 1 {
            info!("Erasing flash...");
            if let Err(e) = eol.erase_flash() {
                error!("Error erasing flash: {e}");
                eol.fail_test();
            }

            info!("Getting efuse information...");
            let efuse_json = match eol.get_efuse_json() {
                Ok(j) => j,
                Err(e) => {
                    error!("Error getting efuse data: {e}");
                    eol.fail_test();
                }
            };

            let efuse_data: serde_json::Value = serde_json::from_str(&efuse_json).unwrap();

            let mac = efuse_data["MAC"]["value"].to_string();
            let mac = mac
                .strip_prefix('\"')
                .unwrap()
                .strip_suffix(" (OK)\"")
                .unwrap()
                .to_string()
                .replace(':', "");

            let results_dir = std::path::Path::new("results");

            if !results_dir.exists() {
                fs::create_dir(results_dir).unwrap();
            }

            let filename =
                results_dir.join(format!("serial_{}_mac_{mac}.json", args.serial_number));
            if filename.exists() {
                error!("Test result file for this serial number and mac already exists!!!");
                exit(-1);
            }
            #[derive(Serialize)]
            struct EolData {
                serial: String,
                time: String,
                adc_largest_tolerance: (u32, i32),
                efuse_data: serde_json::Value,
            }

            info!("Saving data to {}...", filename.display());

            fs::write(
                filename,
                serde_json::to_string_pretty(&EolData {
                    serial: args.serial_number,
                    time: chrono::Utc::now().to_string(),
                    adc_largest_tolerance: results.adc_result.unwrap(),
                    efuse_data,
                })
                .unwrap(),
            )
            .unwrap();

            info!("*** BOARD PASS ***");
            exit(0);
        } else {
            error!("*** BOARD FAIL ***");
            eol.fail_test();
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

    pub fn fail_test(&mut self) -> ! {
        #[cfg(not(target_os = "macos"))]
        {
            warn!("Turning PSU off.");
            self.psu
                .all_outputs_off()
                .map_err(|e| {
                    error!("!!! FAILED TO TURN OFF POWER SUPPLY: MANUALLY TURN OFF PSU NOW !!!");
                    error!("---> {e}");
                })
                .ok();
        }

        error!("##### FAIL ######");
        exit(-1);
    }
}

fn main() {
    EolTest::main();
}
