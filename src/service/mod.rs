// Copyright © ByteHeed.  All rights reserved.

extern crate clap;
extern crate slog;
extern crate winapi;


use super::ffi;
use std::process;

mod consts;
mod structs;
mod core;
mod functions;
pub mod command;


pub use self::functions::*;

pub use self::structs::ServiceStatus;
pub use self::core::{WindowsService, ServiceError};