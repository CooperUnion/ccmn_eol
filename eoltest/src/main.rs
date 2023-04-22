use clap::Parser;
use serialport::SerialPort;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use std::{process::exit, thread::sleep, time::Duration};

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
    tester_port: String,
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
        let mut eol = EolTest { psu };
        #[cfg(target_os = "macos")]
        let mut eol = EolTest { tester };

        eol.prepare_esp32();

        info!("Waiting for 10 seconds...");
        sleep(Duration::from_secs(10));

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
        dbg!(&results);
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
            info!("*** BOARD PASS ***");
            exit(0);
        } else {
            error!("*** BOARD FAIL ***");
            exit(-1);
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
