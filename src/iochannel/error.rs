use std::io::Error;

#[derive(Fail, Debug)]
pub enum DeviceError {
    #[fail(display = "Error opening {:?}: {}", _0, _1)]
    Open(String, #[cause] Error),

    #[fail(display = "I/O call {} ({})", _0, _1)]
    IoCall(u32, #[cause] Error),
    // #[fail(display = "{}", _0)]
    // Io(#[cause] io::Error),
}