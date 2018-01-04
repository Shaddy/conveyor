// Copyright © ByteHeed.  All rights reserved.

use super::iochannel::{ Device, IoCtl };

use super::winapi::um::winioctl;

use super::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use super::num::FromPrimitive;
use super::{Access, Action, Range, GuardFlags, ControlGuard, RegionFlags, RegionStatus};

use super::structs::{RawStruct, SE_GET_CURRENT_EPROCESS};

use std::mem;
use std::fmt;

pub const IOCTL_SENTRY_TYPE: u32 = 0xB080;
pub const SE_NT_DEVICE_NAME: &'static str = "\\\\.\\Sentry";

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


#[allow(dead_code)]
#[derive(Debug)]
pub enum PartitionError {
    NotExists,
    UnknownError
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

pub fn current_process(device: &Device) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A27, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let data = SE_GET_CURRENT_EPROCESS::init();
    
    let (ptr, len) = (data.as_ptr(), data.size());

    device.raw_call(control.into(), ptr, len)
                            .expect("Error calling IOCTL_SENTRY_WRITE_PROCESS_MEMORY");

    data.Process
}

pub fn create_partition(device: &Device) -> Result<Channel, String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A00, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let input = Vec::with_capacity(1000);
    let output: Vec<u8> = Vec::with_capacity(1000);

    
    let cursor = device.call(control.into(), Some(input), Some(output))
                            .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");
    
    Ok(unsafe { Channel::from_raw(cursor.into_inner().as_ptr()) })
}

pub fn delete_partition(device: &Device, id: u64) -> Result<(), String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A01, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).expect("delete_partition() - Failed to write partition id into buffer");
    
    let _ = device.call(control.into(), Some(input), Some(vec![]))
                .expect("Error calling IOCTL_SENTRY_DELETE_PARTITION");

    Ok(())

}


pub fn _get_partition_option(device: &Device, id: u64, option: u64) -> Result<u64, PartitionError> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    
    let mut cursor = match device.call(control.into(), Some(input), Some(output)) 
    {
        Err(err) => {
            return match err.raw_os_error() {
                Some(1167) => Err(PartitionError::NotExists),
                Some(_) | None    => {
                    println!("Device::call() - UnknownError {:?}", err);
                    Err(PartitionError::UnknownError)
                }
            }
        },
        Ok(cursor) => cursor
    };

    Ok(cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong"))
}

pub fn _set_partition_option(device: &Device, id: u64, option: u64, value: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A03, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS);


    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    input.write_u64::<LittleEndian>(value).unwrap();
    
    println!("{:?}", PartitionOption::from_u64(option));
    
    let _ = device.call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_SETOPTION_PARTITION");

    println!("id: {} | option: {:?} | value: {} ", id, option, value);
}

pub fn register_guard_extended(device: &Device, id: u64, context: u64, filter: u64, flags: GuardFlags, priority: u64, _function: u64) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A10, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(context).unwrap();
    input.write_u64::<LittleEndian>(filter).unwrap();
    input.write_u64::<LittleEndian>(flags.bits() as u64).unwrap();
    input.write_u64::<LittleEndian>(priority).unwrap();
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_REGISTER_GUARD");


    cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong")
}

pub fn register_guard(device: &Device, id: u64) -> Result<u64, String> {
    Ok(register_guard_extended(device, id, 0, 0, GuardFlags::STOPPED, 0, 0))
}

pub fn unregister_guard(device: &Device, id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A11, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("Error unregistering guard");
}

pub fn stop_guard(device: &Device, id: u64) {
    control_guard(device, id, ControlGuard::Stop)
}

pub fn start_guard(device: &Device, id: u64) {
    control_guard(device, id, ControlGuard::Start)
}

fn control_guard(device: &Device, id: u64, action: ControlGuard) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A12, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(action as u64).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("control_guard()");
}

pub fn create_region(device: &Device, partition_id: u64, range: &Range, action: Action, access: Access, weight: Option<usize>) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A20, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(range.base).unwrap();
    input.write_u64::<LittleEndian>(range.limit).unwrap();

    // each regions starts disabled
    input.write_u32::<LittleEndian>(RegionFlags::ENABLED.bits()).unwrap(); // flags

    // access
    input.write_u32::<LittleEndian>(access.bits() as u32).unwrap();

    // action
    input.write_u64::<LittleEndian>(action.bits() as u64).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();

    input.write_u64::<LittleEndian>(weight.unwrap_or(0) as u64).unwrap();

    let output: Vec<u8> = Vec::with_capacity(1000);
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("create_region()");

    cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong")
}

pub fn delete_region(device: &Device, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A21, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("delete_region()");
}

pub fn add_region(device: &Device, guard_id: u64, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A22, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("add_region()");
}

#[allow(dead_code)]
pub fn remove_region(device: &Device, guard_id: u64, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A23, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("remove_region()");
}

#[allow(dead_code)]
pub fn set_state_region(device: &Device, region_id: u64, state: RegionStatus) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A24, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id).unwrap();
    input.write_u64::<LittleEndian>(state as u64).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
            .expect("set_state_region()");
}

#[allow(dead_code)]
pub fn get_info_region(device: &Device, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A25, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let mut cursor = device.call(control.into(), Some(input), None)
                .expect("get_info_region()");

    let region_id = cursor.read_u64::<LittleEndian>().expect("can't get <region_id>");
    let next_entry_offset = cursor.read_u64::<LittleEndian>().expect("can't get <next_entry_offset>");
    let base_address = cursor.read_u64::<LittleEndian>().expect("can't get <base_address>");
    let size = cursor.read_u64::<LittleEndian>().expect("can't get <size>");
    let access_type = cursor.read_u32::<LittleEndian>().expect("can't get <access_type>");
    let flags = cursor.read_u64::<LittleEndian>().expect("can't get <flags>");
    let _action = cursor.read_u64::<LittleEndian>().expect("can't get <action>");
    let _action = cursor.read_u64::<LittleEndian>().expect("can't get <action>");
    let action = cursor.read_u64::<LittleEndian>().expect("can't get <action>");
    let weight = cursor.read_u64::<LittleEndian>().expect("can't get <weight>");
    let context = cursor.read_u64::<LittleEndian>().expect("can't get <context>");
    let guard_count = cursor.read_u64::<LittleEndian>().expect("can't get <guard_count>");

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

}

#[allow(dead_code)]
pub fn enumerate_region(device: &Device, partition_id: u64, guard_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A26, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(guard_id).unwrap();

    let output: Vec<u8> = Vec::with_capacity(8 * 1000); // by default it supports 1000 regions
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("remove_region()");



    let _region_id = cursor.read_u64::<LittleEndian>().expect("can't get <region_id>");
}


#[allow(dead_code)]
pub fn create_patch(device: &Device, partition_id: u64, base_address: u64, patch_range: &Range) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A40, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(base_address).unwrap();
    input.write_u64::<LittleEndian>(patch_range.base).unwrap();
    input.write_u64::<LittleEndian>(patch_range.limit).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();

    let output: Vec<u8> = Vec::with_capacity(1000);
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("create_patch()");

    cursor.read_u64::<LittleEndian>().expect("create_patch() - IOCTL Buffer is wrong")
}

#[allow(dead_code)]
pub fn delete_patch(device: &Device, patch_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A41, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("delete_patch()");
}

#[allow(dead_code)]
pub fn add_patch(device: &Device, guard_id: u64, patch_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A42, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(patch_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("add_patch()");
}

#[allow(dead_code)]
pub fn remove_patch(device: &Device, guard_id: u64, patch_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A43, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(patch_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("remove_patch()");
}

#[allow(dead_code)]
pub fn enable_patch(device: &Device, patch_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A44, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("enable_patch()");
}

#[allow(dead_code)]
pub fn disable_patch(device: &Device, patch_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A45, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("disable_patch()");
}


#[allow(dead_code)]
pub fn get_info_patch(device: &Device, patch_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A46, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );


    let mut input = vec![];

    input.write_u64::<LittleEndian>(patch_id).unwrap();
    
    let mut cursor = device.call(control.into(), Some(input), None)
                .expect("get_info_patch()");

    let patch_id          = cursor.read_u64::<LittleEndian>().expect("can't get <patch_id>");
    let next_entry_offset = cursor.read_u64::<LittleEndian>().expect("can't get <next_entry_offset>");
    let base_address      = cursor.read_u64::<LittleEndian>().expect("can't get <base_address>");
    let patch_address     = cursor.read_u64::<LittleEndian>().expect("can't get <patch_address>");
    let patch_size        = cursor.read_u64::<LittleEndian>().expect("can't get <patch_size>");
    let flags             = cursor.read_u64::<LittleEndian>().expect("can't get <flags>");
    let guard_count       = cursor.read_u64::<LittleEndian>().expect("can't get <guard_count>");

    println!("patch_id: 0x{:08X}", patch_id);
    println!("next_entry_offset: 0x{:08X}", next_entry_offset);
    println!("base_address: 0x{:08X}", base_address);
    println!("patch_address: 0x{:08X}", patch_address);
    println!("patch_size: 0x{:08X}", patch_size);
    println!("flags: 0x{:08X}", flags);
    println!("guard_count: 0x{:08X}", guard_count);
}

#[allow(dead_code)]
pub fn enumerate_patch(device: &Device, partition_id: u64, guard_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A47, winioctl::METHOD_BUFFERED, winioctl::FILE_READ_ACCESS | winioctl::FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(guard_id).unwrap();

    let output: Vec<u8> = Vec::with_capacity(8 * 1000);
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("enumerate_patch()");

    let _patch_id = cursor.read_u64::<LittleEndian>().expect("can't get <patch_id>");
}
