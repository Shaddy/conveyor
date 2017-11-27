// Copyright © ByteHeed.  All rights reserved.

use super::kernel32;

use std::ptr::{null_mut};
use std::io::Cursor;
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


#[derive(Debug, Clone)]
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

    pub fn call(&self, control: u32, input: Option<Vec<u8>>, output: Option<Vec<u8>>) -> Result<Cursor<Vec<u8>>, String> {
        let device = Device::open(&self.name).expect("Open device error");

        let mut bytes = 0;
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        // if there is no input, just put a null pointer and 0 as size
        // a little "hack" is that we should remain a reference of input to avoid release the buffer
        // probably it would require some lifetime specification
        let (input_ptr, input_size, _input) = match input {
            Some(mut buffer) => (buffer.as_mut_ptr() as LPVOID, buffer.len() as u32, buffer),
            None => (null_mut(), 0u32, vec![])
        };
                
        // I don't like this at all, but this is what I've went on so far.
        let (output_ptr, output_size, mut output) = match output {
            Some(mut buffer) => (buffer.as_mut_ptr() as LPVOID, buffer.capacity() as u32, buffer),
            None => (null_mut(), 0u32, vec![])
        };

        let success = unsafe {
            kernel32::DeviceIoControl(
                device,
                control,
                input_ptr,
                input_size,
                output_ptr,
                output_size,
                &mut bytes,
                &mut overlapped) != 0
        };

        match success {
            true => {
                unsafe { output.set_len(bytes as usize) };
                output.shrink_to_fit();
                return Ok(Cursor::new(output))
            },
            false => {
                return Err(Error::last_os_error().to_string())
            }
        }
    }
}

