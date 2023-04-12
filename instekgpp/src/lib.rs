use serialport::SerialPort;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No power supply was found.")]
    NoDeviceFound,
    #[error("Error writing to power supply: {0}")]
    WriteError(String),
    #[error("Failed to open power supply: {0}")]
    OpenError(String),
    #[error("Voltage {0}V out of range for channel {1}")]
    VoltageOutOfRange(f64, Channel),
    #[error("Current {0}A out of range for channel {1}")]
    CurrentOutOfRange(f64, Channel),
    #[error("Channel {0} does not support load mode.")]
    ChannelDoesNotSupportLoadMode(Channel)
}

pub struct InstekGpp {
    port: Box<dyn SerialPort>,
}

pub enum Channel {
    C1,
    C2,
    C3,
    C4
}

impl Channel {
    fn to_num(&self) -> u8 {
        match self {
            Channel::C1 => 1,
            Channel::C2 => 2,
            Channel::C3 => 3,
            Channel::C4 => 4,
        }
    }

    fn is_voltage_within_range(&self, voltage: f64) -> bool {
        voltage >= 0.0 && match self {
            Channel::C1 => voltage <= 32.0,
            Channel::C2 => voltage <= 32.0,
            Channel::C3 => voltage <= 5.0,
            Channel::C4 => voltage <= 15.0,
        }
    }

    fn is_current_within_range(&self, current: f64) -> bool {
        current >= 0.0 && match self {
            Channel::C1 => current <= 3.2,
            Channel::C2 => current <= 3.2,
            Channel::C3 => current <= 1.1,
            Channel::C4 => current <= 1.1,
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel {}", self.to_num())
    }
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl InstekGpp {
    pub fn new_first_available() -> Result<InstekGpp, Error> {
        // iterate through serial ports
        for dev in serialport::available_ports().map_err(|_| Error::NoDeviceFound)? {
            eprintln!("device::: {dev:#?}");

            // is it a gpp?
            let serialport::SerialPortType::UsbPort(port) = dev.port_type else {
                continue;
            };

            eprintln!("{:#?}", port.manufacturer);

            if port.vid == 8580 && port.pid == 87 {
                return Ok(InstekGpp {
                    port: serialport::new(dev.port_name, 115200)
                        .open()
                        .map_err(|e| Error::OpenError(e.to_string()))?,
                });
            }
        }

        Err(Error::NoDeviceFound)
    }

    pub fn all_outputs_off(&mut self) -> Result<(), Error> {
        self.port
            .write(":ALLOUTOFF\r\n".as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn all_outputs_on(&mut self) -> Result<(), Error> {
        self.port
            .write(":ALLOUTON\r\n".as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn set_output_voltage(&mut self, channel: Channel, voltage: f64) -> Result<(), Error> {
        if !channel.is_voltage_within_range(voltage) {
            return Err(Error::VoltageOutOfRange(voltage, channel));
        }

        eprintln!("{}", format!(":SOURce{}:VOLTage {:.3}", channel.to_num(), voltage));

        self.port
            .write(format!(":SOURce{}:VOLTage {:.3}\r\n", channel.to_num(), voltage).as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn set_output_current(&mut self, channel: Channel, current: f64) -> Result<(), Error> {
        if !channel.is_current_within_range(current) {
            return Err(Error::CurrentOutOfRange(current, channel));
        }

        eprintln!("{}", format!(":SOURce{}:CURRent {:.3}", channel.to_num(), current));

        self.port
            .write(format!(":SOURce{}:CURRent {:.3}\r\n", channel.to_num(), current).as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn set_load_mode_on(&mut self, channel: Channel) -> Result<(), Error> {
        if matches!(channel, Channel::C3 | Channel::C4) {
            return Err(Error::ChannelDoesNotSupportLoadMode(channel));
        }

        eprintln!("{}", format!(":LOAD{}:CC ON", channel.to_num()));

        self.port
            .write(format!(":LOAD{}:CC ON\r\n", channel.to_num()).as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }

    pub fn set_load_mode_off(&mut self, channel: Channel) -> Result<(), Error> {
        if matches!(channel, Channel::C3 | Channel::C4) {
            return Err(Error::ChannelDoesNotSupportLoadMode(channel));
        }

        eprintln!("{}", format!(":LOAD{}:CC OFF", channel.to_num()));

        self.port
            .write(format!(":LOAD{}:CC OFF\r\n", channel.to_num()).as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{Channel, InstekGpp};

    use anyhow::Result;

    #[test]
    fn test_new_first_available() -> Result<()> {
        let mut psu = InstekGpp::new_first_available()?;

        psu.all_outputs_off()?;

        psu.set_output_current(Channel::C2, 3.1)?;
        psu.set_output_voltage(Channel::C2, 16.0)?;

        psu.set_output_current(Channel::C1, 3.0)?;
        psu.set_load_mode_on(Channel::C1)?;

        psu.all_outputs_on()?;

        sleep(Duration::from_secs(4));

        psu.all_outputs_off()?;
        psu.set_load_mode_off(Channel::C1)?;

        Ok(())
    }
}
