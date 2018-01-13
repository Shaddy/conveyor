use std::io::Error;
use super::IoCtl;

#[derive(Fail, Debug)]
pub enum DeviceError {
    #[fail(display = "Error opening {:?}: {}", _0, _1)]
    Open(String, String),

    #[fail(display = "I/O error calling {}: {}", _0, _1)]
    IoCall(IoCtl, String, #[cause] Error),
    // #[fail(display = "{}", _0)]
    // Io(#[cause] io::Error),
}