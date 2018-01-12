use std::io::Error;

use super::iochannel::error::DeviceError;

#[derive(Fail, Debug)]
pub enum SentryError {
    #[fail(display = "Sentry I/O ({}) ({})", _0, _1)]
    IoCall(String, #[cause] Error),
    #[fail(display = "Error parsing: {} ({})", _0, _1)]
    Parse(String, #[cause] Error),
}

#[derive(Fail, Debug)]
pub enum PartitionError {
    #[fail(display = "Partition {} doesn't exist", _0)]
    NotExists(u64),
    #[fail(display = "UnknownError: {}", _0)]
    UnknownError(#[cause] DeviceError),
}