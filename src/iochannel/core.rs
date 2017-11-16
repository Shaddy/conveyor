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
            // TODO: Build a generic win32 error parser.
            panic!("Invalid handle!!!!");
        }

        Ok( handle )
    }

    pub fn call(&self, control: u32) -> Result<bool, String> {
        let device = Device::open(&self.name).expect("Open device error");

        let mut bytes = 0;
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        // TODO: Create a channel for input/ouput buffers
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

