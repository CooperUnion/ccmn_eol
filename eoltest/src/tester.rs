use std::{
    io::{BufRead, BufReader},
    time::{self, Duration},
};

use anyhow::Result;
use eol_shared::{TestResults, TEST_RESULT_START_MAGIC};
use serialport::SerialPort;
use tracing::info;

use crate::EolTest;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error reading from tester: {0}")]
    ReadError(String),
    #[error("Invalid TestResults serialization: got \"{0}\"")]
    InvalidTestResultsString(String),
    #[error("Timed out waiting for TestResults")]
    TimedOut,
}

macro_rules! port_op {
    ($op: expr, $err: tt) => {
        $op.map_err(|e| Error::$err(e.to_string()))
    };
}

impl EolTest {
    pub fn get_test_result(&self) -> Result<TestResults> {
        let start = time::Instant::now();
        let mut reader = BufReader::new(self.tester.try_clone().unwrap());

        let mut got_first = false;

        while start.elapsed() < Duration::from_secs(10) {
            let mut line = String::new();
            port_op!(reader.read_line(&mut line), ReadError)?;
            info!("Tester output::| {}", line.trim());
            if let Some(results) = line.strip_prefix(TEST_RESULT_START_MAGIC) {
                if !got_first { // skip the first result
                    got_first = true;
                    continue;
                }
                // lol you can use port_op! for this
                let results = port_op!(serde_json::from_str(results), InvalidTestResultsString)?;
                return Ok(results);
            }
        }

        Err(Error::TimedOut.into())
    }
}
