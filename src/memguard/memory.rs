// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::winapi::minwindef::{LPVOID, LPHANDLE};

use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };
use super::kernel32;

use super::byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::Cursor;

use std::mem;

use super::core::IOCTL_SENTRY_TYPE;
use super::iochannel::{Device, IoCtl};
use super::structs;
use super::structs::{RawStruct,
                     SE_MAP_VIRTUAL_MEMORY,
                     SE_UNMAP_VIRTUAL_MEMORY,
                     SE_READ_PROCESS_MEMORY, 
                     SE_WRITE_PROCESS_MEMORY, 
                     SE_ALLOC_VIRTUAL_MEMORY, 
                     SE_FREE_VIRTUAL_MEMORY, 
                     SE_SECURE_VIRTUAL_MEMORY, 
                     SE_UNSECURE_VIRTUAL_MEMORY, 
                     SE_COPY_VIRTUAL_MEMORY, 
                     SE_READ_VIRTUAL_MEMORY, 
                     SE_WRITE_VIRTUAL_MEMORY, 
                     SE_ALLOC_PROCESS_MEMORY, 
                     SE_FREE_PROCESS_MEMORY};

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

pub fn alloc_virtual_memory(device: &Device, size: usize) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A30, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut alloc = SE_ALLOC_VIRTUAL_MEMORY::init();

    alloc.Size = size;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_ALLOC_VIRTUAL_MEMORY");

    
    alloc.BaseAddress as u64
}

pub fn free_virtual_memory(device: &Device, address: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A31, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut alloc = SE_FREE_VIRTUAL_MEMORY::init();

    alloc.BaseAddress = address as LPVOID;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_FREE_VIRTUAL_MEMORY");

}

pub fn copy_virtual_memory(device: &Device, from: u64, to: u64, size: usize) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A33, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut copy = SE_COPY_VIRTUAL_MEMORY::init();

    copy.ToAddress     = to as LPVOID;
    copy.FromAddress   = from as LPVOID;
    copy.Size          = size;

    let (ptr, len) = (copy.as_ptr(), copy.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_COPY_VIRTUAL_MEMORY");

}

pub fn secure_virtual_memory(device: &Device, address: u64, size: usize) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A33, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut secure = SE_SECURE_VIRTUAL_MEMORY::init();

    secure.BaseAddress = address as LPVOID;
    secure.ProbeMode   = 0;

    let (ptr, len) = (secure.as_ptr(), secure.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_SECURE_VIRTUAL_MEMORY");

    secure.SecureHandle as u64

}

pub fn unsecure_virtual_memory(device: &Device, handle: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A34, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut secure = SE_UNSECURE_VIRTUAL_MEMORY::init();

    secure.SecureHandle = handle as LPHANDLE;

    let (ptr, len) = (secure.as_ptr(), secure.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_UNSECURE_VIRTUAL_MEMORY")

}


pub fn map_memory(device: &Device, address: u64, size: usize) -> SE_MAP_VIRTUAL_MEMORY {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A35, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut map = SE_MAP_VIRTUAL_MEMORY::init();

    map.ToProcessId = unsafe { kernel32::GetCurrentProcessId() as u64 };
    map.BaseAddress = address as LPVOID;
    map.Size = size as u32;

    let ptr = map.as_ptr();
    let len = map.size();

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_MAP_MEMORY");

    
    map
}

pub fn unmap_memory(device: &Device, map: SE_MAP_VIRTUAL_MEMORY) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A36, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut unmap = SE_UNMAP_VIRTUAL_MEMORY::init();

    unmap.Mdl = map.Mdl;
    unmap.MappedMemory = map.MappedMemory;

    let (ptr, len) = (unmap.as_ptr(), unmap.size());

    device.raw_call(control.into(), ptr, len)
                        .expect("Error calling IOCTL_SENTRY_UNMAP_MEMORY");
}

// TODO: Evaluate if we should shrink_to_if output vector to BytesCopied
pub fn read_virtual_memory(device: &Device, address: u64, size: usize) -> Vec<u8> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A37, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut read = SE_READ_VIRTUAL_MEMORY::init();

    let mut v: Vec<u8> = vec![0; size];

    read.BaseAddress = address as LPVOID;

    // TODO: Pending to normalize all sizes to usize (SIZE_T)
    read.BytesToRead = size as u32;
    read.Buffer = v.as_mut_ptr() as LPVOID;

    let (ptr, len) = (read.as_ptr(), read.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_READ_VIRTUAL_MEMORY");

    v
}

pub fn write_virtual_memory(device: &Device, address: u64, mut data: Vec<u8>) -> usize {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A38, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut write = SE_WRITE_VIRTUAL_MEMORY::init();

    write.BaseAddress = address as LPVOID;
    write.Buffer = data.as_mut_ptr() as LPVOID;
    write.BytesToWrite = data.len() as u32;

    let (ptr, len) = (write.as_ptr(), write.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_WRITE_VIRTUAL_MEMORY");

    write.BytesCopied as usize
}

pub fn alloc_process_memory(device: &Device, pid: u64, size: usize) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A39, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut alloc = SE_ALLOC_PROCESS_MEMORY::init();

    alloc.ProcessId = pid;
    alloc.BytesToAlloc = size;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_ALLOC_PROCESS_MEMORY");

    alloc.BaseAddress as u64
}

pub fn free_process_memory(device: &Device, pid: u64, address: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A3A, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut alloc = SE_FREE_PROCESS_MEMORY::init();

    alloc.ProcessId = pid;
    alloc.BaseAddress = address as LPVOID;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_FREE_PROCESS_MEMORY");

}

pub fn read_process_memory(device: &Device, pid: u64, address: u64, size: usize) -> Vec<u8> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A3B, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut read = SE_READ_PROCESS_MEMORY::init();

    let v: Vec<u8> = vec![0; size];

    // read.ProcessId = unsafe { kernel32::GetCurrentProcessId() as u64};
    read.ProcessId = pid;
    read.BaseAddress = address as LPVOID;
    read.BytesToRead = size;
    read.Buffer = v.as_ptr() as LPVOID;

    let (ptr, len) = (read.as_ptr(), read.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_READ_PROCESS_MEMORY");

    
    v
}

pub fn write_process_memory(device: &Device, pid: u64, address: u64, mut data: Vec<u8>) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A3C, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut write = SE_WRITE_PROCESS_MEMORY::init();

    write.ProcessId = pid;
    write.BaseAddress = address as LPVOID;
    write.Buffer = data.as_mut_ptr() as LPVOID;
    write.BytesToWrite = data.len();

    let (ptr, len) = (write.as_ptr(), write.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_WRITE_PROCESS_MEMORY");

}


#[allow(dead_code)]
pub fn read_pointer(device: &Device, address: u64) -> u64 {
    read_u64(device, read_u64(device, address))
}


pub fn read_u64(device: &Device, address: u64) -> u64 {
    let v = read_virtual_memory(device, address, 8);

    let mut cursor = Cursor::new(v);

    cursor.read_u64::<LittleEndian>().expect("can't read u64")
}

#[allow(dead_code)]
pub fn read_u32(device: &Device, address: u64) -> u32 {
    let v = read_virtual_memory(device, address, 8);

    let mut cursor = Cursor::new(v);

    cursor.read_u32::<LittleEndian>().expect("can't read u64")
}

#[allow(dead_code)]
pub fn read_u16(device: &Device, address: u64) -> u16 {
    let v = read_virtual_memory(device, address, 8);

    let mut cursor = Cursor::new(v);

    cursor.read_u16::<LittleEndian>().expect("can't read u64")
}
