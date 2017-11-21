// Copyright © ByteHeed.  All rights reserved.

use super::kernel32;

use std::ptr::{null_mut};
use std::io::Error;
// use std::mem::{transmute, zeroed, size_of_val};

use std::mem::{zeroed};

use super::winapi::{HANDLE,
 GENERIC_READ,
 GENERIC_WRITE,
 FILE_SHARE_READ,
 FILE_SHARE_WRITE,
 OPEN_ALWAYS,
 INVALID_HANDLE_VALUE,
 LPVOID};

use super::winapi::minwinbase::{OVERLAPPED};

use ffi::traits::EncodeUtf16;

// bitflags! {
//     pub struct FileAccess: u32 {
//         const FILE_READ_ACCESS  = FILE_READ_ACCESS;
//         const FILE_WRITE_ACCESS = FILE_WRITE_ACCESS;
//     }
// }


#[derive(Debug)]
pub struct IoCtl {
    pub device_type: u32,
    pub function: u32,
    pub method: u32,
    pub access: u32
}

impl IoCtl {
    pub fn new(device_type: u32, function: u32, method: u32, access: u32) -> IoCtl {
        IoCtl {
            device_type: device_type,
            function: function,
            method: method,
            access: access
        }
    }

    pub fn code(&self) -> u32 {
        (self.device_type << 16) |
        (self.access      << 14) |
        (self.function    <<  2) |
         self.method
    }
}

impl Into<u32> for IoCtl {
    fn into(self) -> u32 {
        self.code()
    }
}

impl From<u32> for IoCtl {
    fn from(number: u32) -> IoCtl {
        IoCtl {
            device_type: (number & 0xFFFF0000) >> 16,
            access: (number >> 14) & 3,
            function: (number >> 2) & ((1 << 12) - 1),
            method: number & 3,
        }
    }
}


#[derive(Debug)]
pub enum DeviceError {
    InvalidHandleValue(String)
}

#[derive(Debug)]
pub struct Device {
    name: String
}

impl Device {
    pub fn new(name: &str) -> Device {
        Device {
            name: name.to_string()
        }
    }

    // fn last_error() -> DeviceError {
    //     DeviceError::InvalidHandleValue(Error::last_os_error().to_string()) 
    // }

    pub fn open(name: &str) -> Result<HANDLE, DeviceError> {
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
            return Err(DeviceError::InvalidHandleValue(Error::last_os_error().to_string()) 
)
        }

        Ok( handle )
    }

    pub fn call(&self, control: u32) -> Result<bool, String> {
        let device = Device::open(&self.name).expect("Open device error");

        let mut bytes = 0;
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        println!("IOCTL: 0x{:X}", control);

        let mut input_buffer: Vec<u8> = Vec::with_capacity(1000);
        let mut output_buffer: Vec<u8> = Vec::with_capacity(1000);

        // TODO: Create a channel for input/ouput buffers
        let success = unsafe {
            kernel32::DeviceIoControl(
                device,
                control,
                input_buffer.as_mut_ptr() as LPVOID,
                input_buffer.capacity() as u32,
                output_buffer.as_mut_ptr() as LPVOID,
                output_buffer.capacity() as u32,
                &mut bytes,
                &mut overlapped) == 0
        };

        println!("Device::call (LastError): {}", Error::last_os_error().to_string());
        println!("output_buffer: {:?}", output_buffer);

        Ok(success)
    }
}

