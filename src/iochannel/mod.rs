// Copyright © ByteHeed.  All rights reserved.

extern crate clap;
extern crate slog;
extern crate winapi;

pub mod command;

use self::winapi::um::{ioapiset, fileapi, handleapi};

use std::ptr::{null_mut};
use std::io::Cursor;
use std::io::Error;

use std::mem::{zeroed};

use self::winapi::shared::minwindef::LPVOID;

use self::winapi::um::minwinbase::{OVERLAPPED};
use self::winapi::um::winnt;

use ffi::traits::EncodeUtf16;

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
    name: String,
    device: winnt::HANDLE
}

impl Device {
    pub fn new(name: &str) -> Device {
        let device = Device::open(name).expect("Open device error");

        Device {
            name: name.to_string(),
            device: device
        }
    }

    // fn last_error() -> DeviceError {
    //     DeviceError::InvalidHandleValue(Error::last_os_error().to_string()) 
    // }

    pub fn open(name: &str) -> Result<winnt::HANDLE, DeviceError> {
        let handle = unsafe {
            fileapi::CreateFileW(name.encode_utf16_null().as_ptr(),
                        winnt::GENERIC_READ | winnt::GENERIC_WRITE,
                        winnt::FILE_SHARE_READ | winnt::FILE_SHARE_WRITE,
                        null_mut(),
                        fileapi::OPEN_ALWAYS,
                        0,
                        handleapi::INVALID_HANDLE_VALUE)
        };

        if handle == handleapi::INVALID_HANDLE_VALUE {
            return Err(DeviceError::InvalidHandleValue(Error::last_os_error().to_string()))
        }

        Ok( handle )
    }

    pub fn raw_call(&self, control: u32, ptr: LPVOID, len: usize) -> Result<(), Error> {

        let mut bytes = 0;
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        let success = unsafe {
            ioapiset::DeviceIoControl(
                self.device,
                control,
                ptr,
                len as u32,
                ptr,
                len as u32,
                &mut bytes,
                &mut overlapped) != 0
        };

        match success {
            true => {
                return Ok(())
            },
            false => {
                return Err(Error::last_os_error())
            }
        }
    }

    pub fn call(&self, control: u32, input: Option<Vec<u8>>, output: Option<Vec<u8>>) -> Result<Cursor<Vec<u8>>, Error> {

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
            ioapiset::DeviceIoControl(
                self.device,
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
                return Err(Error::last_os_error())
            }
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            handleapi::CloseHandle(self.device);
        }
    }
}
