use instekgpp::InstekGpp;

use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use std::process::exit;

mod esp32;
mod power;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("CCMN EOL Test ----");

    let mut psu = power::prepare_psu();
    power::check_buck_rails_within_range(&mut psu);
    esp32::prepare_esp32(&mut psu);

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

pub fn fail_test(psu: &mut InstekGpp) -> ! {
    warn!("Turning PSU off.");
    psu.all_outputs_off()
        .map_err(|e| {
            error!("!!! FAILED TO TURN OFF POWER SUPPLY: MANUALLY TURN OFF PSU NOW !!!");
            error!("---> {e}");
        })
        .ok();

    error!("##### FAIL ######");
    exit(-1);
}
