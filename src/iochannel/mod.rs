// Copyright © ByteHeed.  All rights reserved.

extern crate failure;
extern crate clap;
extern crate slog;
extern crate winapi;
extern crate console;

pub mod command;
pub mod error;

use std::fmt;
use self::winapi::um::{ioapiset, fileapi, handleapi, winioctl};

use std::ptr::{null_mut};

use std::io::{Cursor, Error};

use self::error::DeviceError;

use std::mem::{zeroed};

use self::winapi::shared::minwindef::LPVOID;

use self::winapi::um::minwinbase::{OVERLAPPED};
use self::winapi::um::winnt;

use super::cli;

use ffi::traits::EncodeUtf16;

#[derive(Clone)]
pub struct IoCtl {
    name: String,
    pub device_type: u32,
    pub function: u32,
    pub method: u32,
    pub access: u32
}

impl IoCtl {
    pub fn new(name: Option<&str>, device_type: u32, function: u32, method: Option<u32>, access: Option<u32>) -> IoCtl {
        IoCtl {
            name: name.unwrap_or(&format!("0x{:03X}", function)).to_string(),
            device_type: device_type,
            function: function,
            method: method.unwrap_or(winioctl::METHOD_BUFFERED),
            access: access.unwrap_or(winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS)
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
        let function = (number >> 2) & ((1 << 12) - 1);

        IoCtl {
            name: format!("0x{:03X}", function),
            device_type: (number & 0xFFFF_0000) >> 16,
            function: function,
            access: (number >> 14) & 3,
            method: number & 3,
        }
    }
}

impl fmt::Display for IoCtl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for IoCtl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IoCtl{{ name: {}, device: 0x{:X}, function: 0x{:03X} }}", self.name, self.device_type, self.function)
    }
}

#[derive(Debug)]
pub struct Device {
    name: String,
    device: winnt::HANDLE
}

impl Device {
    pub fn new(name: &str) -> Result<Device, DeviceError> {
        let device = Device::open(name)?;

        Ok(
            Device {
            name: name.to_string(),
            device: device
        })
    }

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
            return Err(DeviceError::Open(name.to_string(), Error::last_os_error().to_string()))
        }

        Ok( handle )
    }

    pub fn raw_call(&self, control: IoCtl, ptr: LPVOID, len: usize) -> Result<(), DeviceError> {

        let mut bytes = 0;
        let mut overlapped: OVERLAPPED = unsafe { zeroed() };

        let success = unsafe {
            ioapiset::DeviceIoControl(
                self.device,
                control.code(),
                ptr,
                len as u32,
                ptr,
                len as u32,
                &mut bytes,
                &mut overlapped) != 0
        };

        if !success { return Err(DeviceError::IoCall(control,
                                                     Error::last_os_error().to_string(),
                                                     Error::last_os_error()))};

        Ok(())
    }

    pub fn call(&self, control: IoCtl, input: Option<Vec<u8>>, output: Option<Vec<u8>>) -> Result<Cursor<Vec<u8>>, DeviceError> {

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
                control.code(),
                input_ptr,
                input_size,
                output_ptr,
                output_size,
                &mut bytes,
                &mut overlapped) != 0
        };


        if !success { return Err(DeviceError::IoCall(control,
                                                     Error::last_os_error().to_string(),
                                                     Error::last_os_error()))};

        unsafe { output.set_len(bytes as usize) };
        output.shrink_to_fit();

        Ok(Cursor::new(output))
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            handleapi::CloseHandle(self.device);
        }
    }
}
