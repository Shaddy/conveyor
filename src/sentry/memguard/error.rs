use super::failure;
use super::iochannel::IoCtl;

use std::io::Error;

#[derive(Fail, Debug)]
pub enum MemguardError {
    #[fail(display = "IOCTL ({}) error: ({})", _0, _1)]
    Io(String, #[cause] Error),
}
