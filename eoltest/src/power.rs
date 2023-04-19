use std::{ops::Range, process::exit, thread::sleep, time::Duration};

use anyhow::Result;
use instekgpp::{Channel, InstekGpp};
use tracing::{error, info, warn};

const OK_3V3_RANGE: Range<f64> = 3.27..3.35;
const OK_5V0_RANGE: Range<f64> = 4.98..5.02;

pub fn check_buck_rails_within_range(psu: &mut InstekGpp) -> bool {
    let (v_3v3, v_5v0) = match get_rail_voltages(psu) {
        Ok(v) => v,
        Err(e) => {
            error!("Error while reading rail voltages: {e}");
            return false;
        }
    };

    if !OK_3V3_RANGE.contains(&v_3v3) {
        error!(
            "~~3v3 OUT OF RANGE~~: acceptable is {:?}, actual was {:.2}",
            OK_3V3_RANGE, v_3v3
        );
        return false;
    }

    if !OK_5V0_RANGE.contains(&v_5v0) {
        error!(
            "~~5v0 OUT OF RANGE~~: acceptable is {:?}, actual was {:.2}",
            OK_5V0_RANGE, v_5v0
        );
        return false;
    }

    true
}

fn get_rail_voltages(psu: &mut InstekGpp) -> Result<(f64, f64)> {
    let v_3v3 = psu.measure_voltage(Channel::C1)?;
    let v_5v0 = psu.measure_voltage(Channel::C2)?;

    Ok((v_3v3, v_5v0))
}

pub fn prepare_psu() -> InstekGpp {
    info!("Attaching to power supply...");
    let mut psu = match InstekGpp::new_first_available() {
        Ok(psu) => psu,
        Err(e) => {
            error!("Could not attach to power supply: {e}");
            exit(-1);
        }
    };

    warn!("Configuring and enabling power supply...");
    match configure_psu_settings(&mut psu) {
        Ok(()) => {
            info!("Waiting for power supply to stabilize.");
            sleep(Duration::from_secs(4));
            info!("Power supply ready.");
        }
        Err(e) => {
            error!("Failed to prepare power supply: {e}");
            exit(-1);
        }
    };

    psu
}

fn configure_psu_settings(psu: &mut InstekGpp) -> Result<()> {
    psu.all_outputs_off()?;

    psu.set_output_voltage(Channel::C4, 15.0)?;
    psu.set_output_current(Channel::C4, 1.1)?;

    psu.set_output_voltage(Channel::C1, 0.0)?;
    psu.set_output_current(Channel::C1, 0.0)?;
    psu.set_load_mode_on(Channel::C1)?;

    psu.set_output_voltage(Channel::C2, 0.0)?;
    psu.set_output_current(Channel::C2, 0.0)?;
    psu.set_load_mode_on(Channel::C2)?;

    psu.all_outputs_on()?;

    Ok(())
}
