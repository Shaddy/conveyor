// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::winapi::shared::minwindef::{LPVOID, LPHANDLE};

use super::winapi::um::{ processthreadsapi, winioctl };

use super::byteorder::{LittleEndian, ReadBytesExt};

use std::marker::PhantomData;
use std::io::Cursor;

use std::slice;
use std::mem;

use super::failure::Error;
use super::io::IOCTL_SENTRY_TYPE;
use super::iochannel::{Device, IoCtl};
use super::structs;

pub use super::structs::MapMode;

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
pub struct KernelAlloc<'a, T> {
    device: &'a Device,
    map: Map<'a>,
    phantom: PhantomData<T>
}

impl<'a, T> KernelAlloc<'a, T> {
    pub fn new(device: &'a Device) -> KernelAlloc<'a, T> {
        let size = mem::size_of::<T>();
        let ptr = alloc_virtual_memory(device, size)
                        .expect("failed to allocate memory");

        // memset
        let v: Vec<u8> = vec![0; size];
        write_virtual_memory(device, ptr, v).expect("write memory");


        KernelAlloc {
            device: device,
            map: Map::new(device, ptr, size, Some(MapMode::UserMode)),
            phantom: PhantomData
        }
    }

    pub fn size(&self) -> usize {
        mem::size_of::<T>()
    }

    pub fn kernel_ptr(&self) -> u64 {
        self.map.kernel_ptr()
    }

    #[allow(dead_code)]
    pub fn as_slice(&self) -> &[u8] {
        self.map.as_slice()
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.map.raw.MappedMemory as *mut u8 as *mut T
    }

    pub fn as_ptr(&self) -> *const T {
        self.map.raw.MappedMemory as *const u8 as *const T
    }
}

impl<'a, T> Drop for KernelAlloc<'a, T> {
    fn drop(&mut self) {
        free_virtual_memory(self.device, self.map.kernel_ptr()).expect("free error");
    }
}

#[derive(Debug)]
pub struct Map<'a> {
    device: &'a Device,
    address: u64,
    size: usize,
    raw: structs::SE_MAP_VIRTUAL_MEMORY
}

impl<'a> Map<'a> {
    pub fn new(device: &'a Device, address: u64, size: usize, mode: Option<MapMode>) -> Map<'a> {
        let raw = map_memory(device, address, size, mode)
                            .expect("failed to map memory");

        Map {
            device: device,
            address: address,
            size: size,
            raw: raw
        }
    }

    pub fn kernel_ptr(&self) -> u64 {
        self.raw.BaseAddress as u64
    }

    #[allow(dead_code)]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.raw.MappedMemory as *mut u8
    }

    #[allow(dead_code)]
    pub fn as_ptr(&self) -> *const u8 {
        self.raw.MappedMemory as *const u8
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.raw.MappedMemory as *const u8, self.size) }
    }
}

impl<'a> Drop for Map<'a> {
    fn drop(&mut self) {
        unmap_memory(self.device, self.raw)
                    .expect("unmap error");
    }
}

pub fn alloc_virtual_memory(device: &Device, size: usize) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A50, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut alloc = SE_ALLOC_VIRTUAL_MEMORY::init();

    alloc.Size = size;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)?;

    
    Ok(alloc.BaseAddress as u64)
}

pub fn free_virtual_memory(device: &Device, address: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A51, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut alloc = SE_FREE_VIRTUAL_MEMORY::init();

    alloc.BaseAddress = address as LPVOID;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(())

}

#[allow(dead_code)]
pub fn copy_virtual_memory(device: &Device, from: u64, to: u64, size: usize) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A52, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut copy = SE_COPY_VIRTUAL_MEMORY::init();

    copy.ToAddress     = to as LPVOID;
    copy.FromAddress   = from as LPVOID;
    copy.Size          = size;

    let (ptr, len) = (copy.as_ptr(), copy.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(())

}

#[allow(dead_code)]
pub fn secure_virtual_memory(device: &Device, address: u64, size: usize) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A53, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut secure = SE_SECURE_VIRTUAL_MEMORY::init();

    secure.BaseAddress = address as LPVOID;
    secure.Size        = size;
    secure.ProbeMode   = 0;

    let (ptr, len) = (secure.as_ptr(), secure.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(secure.SecureHandle as u64)

}

#[allow(dead_code)]
pub fn unsecure_virtual_memory(device: &Device, handle: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A54, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut secure = SE_UNSECURE_VIRTUAL_MEMORY::init();

    secure.SecureHandle = handle as LPHANDLE;

    let (ptr, len) = (secure.as_ptr(), secure.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(())

}

pub fn map_memory(device: &Device, address: u64, size: usize, mode: Option<MapMode>) -> Result<SE_MAP_VIRTUAL_MEMORY, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A55, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut map = SE_MAP_VIRTUAL_MEMORY::init();

    map.ToProcessId = u64::from(unsafe { processthreadsapi::GetCurrentProcessId()});
    map.BaseAddress = address as LPVOID;
    map.MapMode = mode.unwrap_or(MapMode::UserMode);
    map.Size = size as u32;

    let ptr = map.as_ptr();
    let len = map.size();

    device.raw_call(control.into(), ptr, len)?;

    
    Ok(map)
}

pub fn unmap_memory(device: &Device, map: SE_MAP_VIRTUAL_MEMORY) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A56, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut unmap = SE_UNMAP_VIRTUAL_MEMORY::init();

    unmap.Mdl = map.Mdl;
    unmap.MappedMemory = map.MappedMemory;

    let (ptr, len) = (unmap.as_ptr(), unmap.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(())
}

// TODO: Evaluate if we should shrink_to_if output vector to BytesCopied
pub fn read_virtual_memory(device: &Device, address: u64, size: usize) -> Result<Vec<u8>, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A57, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut read = SE_READ_VIRTUAL_MEMORY::init();

    let mut v: Vec<u8> = vec![0; size];

    read.BaseAddress = address as LPVOID;

    // TODO: Pending to normalize all sizes to usize (SIZE_T)
    read.BytesToRead = size as u32;
    read.Buffer = v.as_mut_ptr() as LPVOID;

    let (ptr, len) = (read.as_ptr(), read.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(v)
}

pub fn write_virtual_memory(device: &Device, address: u64, mut data: Vec<u8>) -> Result<usize, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A58, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut write = SE_WRITE_VIRTUAL_MEMORY::init();

    write.BaseAddress = address as LPVOID;
    write.Buffer = data.as_mut_ptr() as LPVOID;
    write.BytesToWrite = data.len() as u32;

    let (ptr, len) = (write.as_ptr(), write.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(write.BytesCopied as usize)
}

#[allow(dead_code)]
pub fn alloc_process_memory(device: &Device, pid: u64, size: usize) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A59, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut alloc = SE_ALLOC_PROCESS_MEMORY::init();

    alloc.ProcessId = pid;
    alloc.BytesToAlloc = size;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(alloc.BaseAddress as u64)
}

#[allow(dead_code)]
pub fn free_process_memory(device: &Device, pid: u64, address: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A5A, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut alloc = SE_FREE_PROCESS_MEMORY::init();

    alloc.ProcessId = pid;
    alloc.BaseAddress = address as LPVOID;

    let (ptr, len) = (alloc.as_ptr(), alloc.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(())

}

#[allow(dead_code)]
pub fn read_process_memory(device: &Device, pid: u64, address: u64, size: usize) -> Result<Vec<u8>, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A5B, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut read = SE_READ_PROCESS_MEMORY::init();

    let v: Vec<u8> = vec![0; size];

    // read.ProcessId = unsafe { kernel32::GetCurrentProcessId() as u64};
    read.ProcessId = pid;
    read.BaseAddress = address as LPVOID;
    read.BytesToRead = size;
    read.Buffer = v.as_ptr() as LPVOID;

    let (ptr, len) = (read.as_ptr(), read.size());

    device.raw_call(control.into(), ptr, len)?;

    
    Ok(v)
}

#[allow(dead_code)]
pub fn write_process_memory(device: &Device, pid: u64, address: u64, mut data: Vec<u8>) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A5C, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut write = SE_WRITE_PROCESS_MEMORY::init();

    write.ProcessId = pid;
    write.BaseAddress = address as LPVOID;
    write.Buffer = data.as_mut_ptr() as LPVOID;
    write.BytesToWrite = data.len();

    let (ptr, len) = (write.as_ptr(), write.size());

    device.raw_call(control.into(), ptr, len)?;

    Ok(())
}

#[allow(dead_code)]
pub fn read_pointer(device: &Device, address: u64) -> Result<u64, Error> {
    read_u64(device, read_u64(device, address)?)
}

pub fn read_u64(device: &Device, address: u64) -> Result<u64, Error> {
    let v = read_virtual_memory(device, address, 8)?;

    let mut cursor = Cursor::new(v);

    Ok(cursor.read_u64::<LittleEndian>()?)
}

#[allow(dead_code)]
pub fn read_u32(device: &Device, address: u64) -> Result<u32, Error> {
    let v = read_virtual_memory(device, address, 8)?;

    let mut cursor = Cursor::new(v);

    Ok( cursor.read_u32::<LittleEndian>()? )
}

#[allow(dead_code)]
pub fn read_u16(device: &Device, address: u64) -> Result<u16, Error> {
    let v = read_virtual_memory(device, address, 8)?;

    let mut cursor = Cursor::new(v);

    Ok( cursor.read_u16::<LittleEndian>()? )
}
