// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::winapi::minwindef::{LPVOID};

use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };
use super::kernel32;

use super::core::IOCTL_SENTRY_TYPE;
use super::iochannel::{Device, IoCtl};
use super::structs;
use super::structs::{RawStruct,
                     SE_MAP_VIRTUAL_MEMORY, 
                     SE_UNMAP_VIRTUAL_MEMORY,
                     SE_READ_PROCESS_MEMORY,
                     SE_WRITE_PROCESS_MEMORY};

#[derive(Debug)]
pub struct Map<'a> {
    device: &'a Device,
    address: u64,
    size: usize,
    raw: structs::SE_MAP_VIRTUAL_MEMORY
}

impl<'a> Map<'a> {
    pub fn new(device: &'a Device, address: u64, size: usize) -> Map<'a> {
        let raw = map_memory(&device, address, size);

        Map {
            device: device,
            address: address,
            size: size,
            raw: raw
        }
    }
}

impl<'a> Drop for Map<'a> {
    fn drop(&mut self) {
        unmap_memory(self.device, self.raw);
    }
}

pub fn map_memory(device: &Device, address: u64, size: usize) -> SE_MAP_VIRTUAL_MEMORY {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A30, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut map = SE_MAP_VIRTUAL_MEMORY::init();

    map.ProcessId = unsafe { kernel32::GetCurrentProcessId() as u64 };
    map.BaseAddress = address as LPVOID;
    map.Size = size as u32;

    let ptr = map.as_ptr();
    let len = map.size();

    device.raw_call(control.into(), ptr, len, ptr, len)
                            .expect("Error calling IOCTL_SENTRY_MAP_MEMORY");

    
    map
}

pub fn unmap_memory(device: &Device, map: SE_MAP_VIRTUAL_MEMORY) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A31, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut unmap = SE_UNMAP_VIRTUAL_MEMORY::init();

    unmap.Mdl = map.Mdl;
    unmap.MappedMemory = map.MappedMemory;

    let ptr = unmap.as_ptr();
    let len = unmap.size();

    device.raw_call(control.into(), ptr, len, ptr, len)
                        .expect("Error calling IOCTL_SENTRY_UNMAP_MEMORY");
}

pub fn read_memory(device: &Device, address: u64, size: usize) -> Vec<u8> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A32, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut read = SE_READ_PROCESS_MEMORY::init();

    let v: Vec<u8> = vec![0; size];

    read.ProcessId = unsafe { kernel32::GetCurrentProcessId() as u64};
    read.BaseAddress = address as LPVOID;
    read.BytesToRead = size;
    read.Buffer = v.as_ptr() as LPVOID;

    let ptr = read.as_ptr();
    let len = read.size();

    device.raw_call(control.into(), ptr, len, ptr, len)
                            .expect("Error calling IOCTL_SENTRY_READ_MEMORY");

    
    v
}

pub fn write_memory(device: &Device, address: u64, mut data: Vec<u8>) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A33, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut write = SE_WRITE_PROCESS_MEMORY::init();

    write.ProcessId = unsafe { kernel32::GetCurrentProcessId() as u64 };
    write.BaseAddress = address as LPVOID;
    write.Buffer = data.as_mut_ptr() as LPVOID;
    write.BytesToWrite = data.len();

    let ptr = write.as_ptr();
    let len = write.size();

    device.raw_call(control.into(), ptr, len, ptr, len)
                            .expect("Error calling IOCTL_SENTRY_MAP_MEMORY");

}
