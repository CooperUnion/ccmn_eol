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
    VoltageOutOfRange(f64, Channel)
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
            Channel::C1 => voltage <= 15.0,
            Channel::C2 => voltage <= 32.0,
            Channel::C3 => voltage <= 32.0,
            Channel::C4 => voltage <= 5.0,
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

        self.port
            .write(format!(":SOURce{}:VOLTage {:.3}", channel.to_num(), voltage).as_bytes())
            .map_err(|e| Error::WriteError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Channel;

    #[test]
    fn test_new_first_available() {
        let mut psu = crate::InstekGpp::new_first_available().unwrap();

        psu.all_outputs_off().unwrap();
        psu.set_output_voltage(Channel::C1, 3.0).unwrap();
        psu.all_outputs_on().unwrap();
    }
}
