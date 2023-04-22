use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use std::process::exit;

mod esp32;

cfg_if::cfg_if! {
    if #[cfg(not(target_os = "macos"))] {
        use instekgpp::InstekGpp;
        mod power;
    }
}

struct EolTest {
    #[cfg(not(target_os = "macos"))]
    psu: InstekGpp,
}

impl EolTest {
    fn main() {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        info!("CCMN EOL Test ----");

        #[cfg(not(target_os = "macos"))]
        let psu = power::prepare_psu();

        #[cfg(not(target_os = "macos"))]
        let mut eol = EolTest { psu };
        #[cfg(target_os = "macos")]
        let mut eol = EolTest {};

        eol.prepare_esp32();

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
