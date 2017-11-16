// Copyright © ByteHeed.  All rights reserved.

use super::kernel32;

use std::ptr::{null_mut};
use std::mem::{transmute, zeroed, size_of_val};

use super::winapi::{HANDLE,
 GENERIC_READ,
 GENERIC_WRITE,
 FILE_SHARE_READ,
 FILE_SHARE_WRITE,
 OPEN_ALWAYS,
 INVALID_HANDLE_VALUE};

use super::winapi::minwinbase::{OVERLAPPED};


use ffi::traits::EncodeUtf16;

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;


pub fn device_subcommand() -> App<'static, 'static> {
    SubCommand::with_name("device").about("tests all device related functionality")
}

pub struct Device {
    name: String
}

impl Device {
    pub fn new(name: &str) -> Device {
        Device {
            name: name.to_string()
        }
    }

    fn open(name: &str) -> Result<HANDLE, ()> {
        let handle = unsafe {
            kernel32::CreateFileW(name.encode_utf16_null().as_ptr(),
                        GENERIC_READ | GENERIC_WRITE,
                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                        null_mut(),
                        OPEN_ALWAYS,
                        0,
                        INVALID_HANDLE_VALUE)
        };

        if handle == INVALID_HANDLE_VALUE {
            panic!("Invalid handle!!!!");
        }

        Ok( handle )
    }

    pub fn call(&self, control: u32) -> Result<bool, String> {
        let device = Device::open(&self.name).expect("Open device error");

        let mut bytes = 0;
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        let success = unsafe {
            kernel32::DeviceIoControl(
                device,
                control,
                null_mut(),
                0,
                null_mut(),
                0,
                &mut bytes,
                &mut overlapped) == 0
        };

        Ok(success)
    }
}


pub fn iochannel_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("open")  => open_device(logger),
        _             => println!("{}", matches.usage())
    }
}

fn open_device(logger: Logger) {
    Device::new("/devices/memguard").call(0);
}