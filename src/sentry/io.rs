// Copyright © ByteHeed.  All rights reserved.

use super::iochannel::{ Device, IoCtl };

use super::winapi::um::winioctl;

use super::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use super::memguard::{Access, Action, Range, GuardFlags, ControlGuard, RegionFlags, RegionStatus, Filter};

use super::misc;
use super::iochannel::error::DeviceError;
use super::error::PartitionError;
use super::failure::Error;
use std::io::Cursor;

use self::misc::Process;

use std::{mem, fmt};

pub const IOCTL_SENTRY_TYPE: u32 = 0xB080;
pub const SE_NT_DEVICE_NAME: &str = "\\\\.\\Sentry";

enum_from_primitive! {
    #[derive(Debug, Clone)]
    enum PartitionOption {
        None = 0,
        TraceDebugEvents,
        TraceToDisk,
        CoalesceNotifications,
        CollectStats,
        SecureMode,
    }
}



#[repr(C)]
pub struct Channel {
    pub id: u64,
    pub address: u64,
    pub size: u32
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Channel(id: 0x{:016X}, address: 0x{:016x}, size: 0x{:016x}", 
                        self.id,
                        self.address,
                        self.size)
    }
}

impl Channel {
    pub unsafe fn from_raw(ptr: *const u8) -> Channel {
        mem::transmute_copy(&*ptr)
    }
}

pub fn create_partition(device: &Device) -> Result<Channel, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A00, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let input = Vec::with_capacity(1000);
    let output: Vec<u8> = Vec::with_capacity(1000);

    
    let cursor = device.call(control.into(), Some(input), Some(output))?;

    Ok(unsafe { Channel::from_raw(cursor.into_inner().as_ptr()) })
}

pub fn delete_partition(device: &Device, id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A01, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);

    let mut input = vec![];

    input.write_u64::<LittleEndian>(id)?;
    
    device.call(control.into(), Some(input), Some(vec![0; 1024]))?;

    Ok(())
}

fn partition_result(id: u64, result: Result<Cursor<Vec<u8>>, DeviceError>) -> Result<Cursor<Vec<u8>>, PartitionError> {
    match result {
        Err(err) => {
            if let DeviceError::IoCall(n, s, io_err) = err {
                if let Some(1167) = io_err.raw_os_error() {
                    return Err(PartitionError::NotExists(id))
                } else {
                    return Err(PartitionError::UnknownError(DeviceError::IoCall(n, s, io_err)));
                } 
            }  else {
                return Err(PartitionError::UnknownError(err));
            }
                    
        },
        Ok(cursor) => Ok(cursor)
    }
}

#[allow(dead_code)]
pub fn get_partition_option(device: &Device, id: u64, option: u64) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id)?;
    input.write_u64::<LittleEndian>(option)?;

    let mut cursor = partition_result(id, device.call(control.into(), Some(input), Some(output)))?;

    Ok(cursor.read_u64::<LittleEndian>()?)
}

#[allow(dead_code)]
pub fn set_partition_option(device: &Device, id: u64, option: u64, value: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A03, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id)?;
    input.write_u64::<LittleEndian>(option)?;
    input.write_u64::<LittleEndian>(value)?;
    
    let _ = device.call(control.into(), Some(input), Some(output))?;

    Ok(())
}

pub fn register_guard_extended(device: &Device, id: u64, process: Option<Process>, filter: Option<Filter>, flags: GuardFlags, priority: u64, _function: u64) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A10, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    // its important to keep filter lifetime until the end
    // the device call to avoid dropping the internal allocation
    let (ptr, _filter) = if let Some(filter) = filter { (filter.kernel_ptr(), Some(filter)) } else { (0, None) };

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    let eprocess = if let Some(process) = process { process.object() } else { 0 };

    input.write_u64::<LittleEndian>(id)?;
    input.write_u64::<LittleEndian>(eprocess)?;
    input.write_u64::<LittleEndian>(ptr)?;
    input.write_u64::<LittleEndian>(u64::from(flags.bits()))?;
    input.write_u64::<LittleEndian>(priority)?;
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))?;

    Ok(cursor.read_u64::<LittleEndian>()?)
}

pub fn register_guard(device: &Device, id: u64, filter: Option<Filter>) -> Result<u64, Error> {
    let current = misc::Process::current();
    Ok(register_guard_extended(device, id, Some(current), filter, GuardFlags::STOPPED, 0, 0)?)
}

pub fn unregister_guard(device: &Device, id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A11, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

pub fn stop_guard(device: &Device, id: u64) -> Result<(), Error> {
    control_guard(device, id, ControlGuard::Stop)?;
    Ok(())
}

pub fn start_guard(device: &Device, id: u64) -> Result<(), Error> {
    control_guard(device, id, ControlGuard::Start)?;
    Ok(())
}

fn control_guard(device: &Device, id: u64, action: ControlGuard) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A12, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(id)?;
    input.write_u64::<LittleEndian>(action as u64)?;
    
    let _ = device.call(control.into(), Some(input), None)?;

    Ok(())
}

pub fn create_region(device: &Device, partition_id: u64, range: &Range, action: Action, access: Access, weight: Option<usize>) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A20, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id)?;
    input.write_u64::<LittleEndian>(range.base)?;
    input.write_u64::<LittleEndian>(range.limit)?;

    // each regions starts disabled
    input.write_u32::<LittleEndian>(RegionFlags::ENABLED.bits())?; // flags

    // access
    input.write_u32::<LittleEndian>(u32::from(access.bits()))?;

    // action
    input.write_u64::<LittleEndian>(u64::from(action.bits()))?;

    // readbuffer
    input.write_u64::<LittleEndian>(0)?;

    // writebuffer
    input.write_u64::<LittleEndian>(0)?;

    input.write_u64::<LittleEndian>(weight.unwrap_or(0) as u64)?;

    let output: Vec<u8> = Vec::with_capacity(1000);
    let mut cursor = device.call(control.into(), Some(input), Some(output))?;

    Ok(cursor.read_u64::<LittleEndian>()?)
}

pub fn delete_region(device: &Device, region_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A21, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

pub fn add_region(device: &Device, guard_id: u64, region_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A22, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id)?;
    input.write_u64::<LittleEndian>(region_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn remove_region(device: &Device, guard_id: u64, region_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A23, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id)?;
    input.write_u64::<LittleEndian>(region_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn set_state_region(device: &Device, region_id: u64, state: RegionStatus) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A24, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id)?;
    input.write_u64::<LittleEndian>(state as u64)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_info_region(device: &Device, region_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A25, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id)?;

    let mut cursor = device.call(control.into(), Some(input), None)?;

    let region_id = cursor.read_u64::<LittleEndian>()?;
    let next_entry_offset = cursor.read_u64::<LittleEndian>()?;
    let base_address = cursor.read_u64::<LittleEndian>()?;
    let size = cursor.read_u64::<LittleEndian>()?;
    let access_type = cursor.read_u32::<LittleEndian>()?;
    let flags = cursor.read_u64::<LittleEndian>()?;
    let _action = cursor.read_u64::<LittleEndian>()?;
    let _action = cursor.read_u64::<LittleEndian>()?;
    let action = cursor.read_u64::<LittleEndian>()?;
    let weight = cursor.read_u64::<LittleEndian>()?;
    let context = cursor.read_u64::<LittleEndian>()?;
    let guard_count = cursor.read_u64::<LittleEndian>()?;

    println!("region_id: 0x{:08X}", region_id);
    println!("base_address: 0x{:08X}", base_address);
    println!("next_entry_offset: 0x{:08X}", next_entry_offset);
    println!("size: 0x{:08X}", size);
    println!("access_type: 0x{:08X}", access_type);
    println!("flags: 0x{:08X}", flags);
    println!("action: 0x{:08X}", action);
    println!("weight: 0x{:08X}", weight);
    println!("context: 0x{:08X}", context);
    println!("guard_count: 0x{:08X}", guard_count);
    println!("region_id: 0x{:08X}", region_id);

    Ok(())
}

#[allow(dead_code)]
pub fn enumerate_region(device: &Device, partition_id: u64, guard_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A26, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id)?;
    input.write_u64::<LittleEndian>(guard_id)?;

    let output: Vec<u8> = Vec::with_capacity(8 * 1000); // by default it supports 1000 regions
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))?;

    let _region_id = cursor.read_u64::<LittleEndian>()?;

    // TODO: result in an enumerate object (iterator over buffer?)
    Ok(())
}


#[allow(dead_code)]
pub fn create_patch(device: &Device, partition_id: u64, base_address: u64, patch_range: &Range) -> Result<u64, Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A40, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id)?;
    input.write_u64::<LittleEndian>(base_address)?;
    input.write_u64::<LittleEndian>(patch_range.base)?;
    input.write_u64::<LittleEndian>(patch_range.limit)?;
    input.write_u64::<LittleEndian>(0)?;

    let output: Vec<u8> = Vec::with_capacity(1000);
    let mut cursor = device.call(control.into(), Some(input), Some(output))?;

    Ok(cursor.read_u64::<LittleEndian>()?)
}

#[allow(dead_code)]
pub fn delete_patch(device: &Device, patch_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A41, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn add_patch(device: &Device, guard_id: u64, patch_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A42, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id)?;
    input.write_u64::<LittleEndian>(patch_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn remove_patch(device: &Device, guard_id: u64, patch_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A43, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id)?;
    input.write_u64::<LittleEndian>(patch_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn enable_patch(device: &Device, patch_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A44, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}

#[allow(dead_code)]
pub fn disable_patch(device: &Device, patch_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A45, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id)?;
    
    let _ = device.call(control.into(), Some(input), None)?;
    Ok(())
}


#[allow(dead_code)]
pub fn get_info_patch(device: &Device, patch_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A46, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id)?;
    
    let mut cursor = device.call(control.into(), Some(input), None)?;

    let patch_id          = cursor.read_u64::<LittleEndian>()?;
    let next_entry_offset = cursor.read_u64::<LittleEndian>()?;
    let base_address      = cursor.read_u64::<LittleEndian>()?;
    let patch_address     = cursor.read_u64::<LittleEndian>()?;
    let patch_size        = cursor.read_u64::<LittleEndian>()?;
    let flags             = cursor.read_u64::<LittleEndian>()?;
    let guard_count       = cursor.read_u64::<LittleEndian>()?;

    println!("patch_id: 0x{:08X}", patch_id);
    println!("next_entry_offset: 0x{:08X}", next_entry_offset);
    println!("base_address: 0x{:08X}", base_address);
    println!("patch_address: 0x{:08X}", patch_address);
    println!("patch_size: 0x{:08X}", patch_size);
    println!("flags: 0x{:08X}", flags);
    println!("guard_count: 0x{:08X}", guard_count);
    Ok(())
}

#[allow(dead_code)]
pub fn enumerate_patch(device: &Device, partition_id: u64, guard_id: u64) -> Result<(), Error> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A47, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id)?;
    input.write_u64::<LittleEndian>(guard_id)?;

    let output: Vec<u8> = Vec::with_capacity(8 * 1000);
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))?;

    let _patch_id = cursor.read_u64::<LittleEndian>()?;
    Ok(())
}
