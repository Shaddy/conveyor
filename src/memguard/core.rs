// Copyright © ByteHeed.  All rights reserved.

use super::iochannel::{ Device, IoCtl };
use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };
use super::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use super::num::FromPrimitive;
use super::{Access, Range, GuardFlags, ControlGuard, RegionFlags, RegionStatus};

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

pub fn create_partition(device: &Device) -> Result<Channel, String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A00, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let input = Vec::with_capacity(1000);
    let output: Vec<u8> = Vec::with_capacity(1000);

    
    let cursor = device.call(control.into(), Some(input), Some(output))
                            .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");
    
    Ok(unsafe { Channel::from_raw(cursor.into_inner().as_ptr()) })
}

pub fn delete_partition(device: &Device, id: u64) -> Result<(), String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A01, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).expect("delete_partition() - Failed to write partition id into buffer");
    
    let _ = device.call(control.into(), Some(input), Some(vec![]))
                .expect("Error calling IOCTL_SENTRY_DELETE_PARTITION");

    Ok(())

}


pub fn _get_partition_option(device: &Device, id: u64, option: u64) -> Result<u64, PartitionError> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

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
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A03, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

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
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A10, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

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
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A11, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

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
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A12, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(action as u64).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("control_guard()");
}

pub fn create_region(device: &Device, partition_id: u64, range: &Range, access: Access, weight: Option<usize>) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A20, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(range.base).unwrap();
    input.write_u64::<LittleEndian>(range.limit).unwrap();

    // each regions starts disabled
    input.write_u32::<LittleEndian>(RegionFlags::ENABLED.bits()).unwrap(); // flags

    // access
    input.write_u32::<LittleEndian>(access.bits() as u32).unwrap();

    // action
    input.write_u64::<LittleEndian>(0x0008 | 0x1000).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();

    input.write_u64::<LittleEndian>(weight.unwrap_or(0) as u64).unwrap();

    let output: Vec<u8> = Vec::with_capacity(1000);
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("create_region()");

    cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong")
}

pub fn delete_region(device: &Device, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A21, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("delete_region()");
}

pub fn add_region(device: &Device, guard_id: u64, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A22, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("add_region()");
}

pub fn remove_region(device: &Device, guard_id: u64, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A23, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
                .expect("remove_region()");
}

pub fn _set_state_region(device: &Device, region_id: u64, state: RegionStatus) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A24, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id).unwrap();
    input.write_u64::<LittleEndian>(state as u64).unwrap();
    
    let _ = device.call(control.into(), Some(input), None)
            .expect("set_state_region()");
}

pub fn _get_info_region(device: &Device, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A25, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

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

pub fn _enumerate_region(device: &Device, partition_id: u64, guard_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A26, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(guard_id).unwrap();

    let output: Vec<u8> = Vec::with_capacity(8 * 1000); // by default it supports 1000 regions
    
    let mut cursor = device.call(control.into(), Some(input), Some(output))
                .expect("remove_region()");



    let _region_id = cursor.read_u64::<LittleEndian>().expect("can't get <region_id>");
}

// In an ideal scenario this should be real consts
// 
// const IOCTL_SENTRY_CREATE_PARTITION: IoCtl =    IoCtl::new( 0x0A00, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);
// const IOCTL_SENTRY_DELETE_PARTITION: IoCtl =    IoCtl::new( 0x0A01, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_GETOPTION_PARTITION: IoCtl = IoCtl::new( 0x0A02, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_SETOPTION_PARTITION: IoCtl = IoCtl::new( 0x0A03, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_REGISTER_GUARD: IoCtl =      IoCtl::new( 0x0A10, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_UNREGISTER_GUARD: IoCtl =    IoCtl::new( 0x0A11, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_CONTROL_GUARD: IoCtl =       IoCtl::new( 0x0A12, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_CREATE_REGION: IoCtl =       IoCtl::new( 0x0A20, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_DELETE_REGION: IoCtl =       IoCtl::new( 0x0A21, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_ADD_REGION: IoCtl =          IoCtl::new( 0x0A22, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_REMOVE_REGION: IoCtl =       IoCtl::new( 0x0A23, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_SET_STATE_REGION: IoCtl =    IoCtl::new( 0x0A24, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_GET_INFO_REGION: IoCtl =     IoCtl::new( 0x0A25, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
// const IOCTL_SENTRY_ENUMERATE_REGION: IoCtl =    IoCtl::new( 0x0A26, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

// use std::ops::Fn;
// struct SentryChannel<FS, FD> {
//     control: IoCtl,
//     device: &Device,
//     deserializer: Option<FD>,
//     serializer: Option<FS>
// }

// impl<FS, FD> SentryChannel<FS, FD>
//     where FD: Fn(&mut Cursor<Vec<u8>>) -> u64,
//           FS: Fn(&mut Vec<u8>) { 
//     fn new(function: usize) -> Self {
//         SentryChannel {
//             control: IoCtl::new(IOCTL_SENTRY_TYPE, function as u32, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS),
//             device: &Device::new(SE_NT_DEVICE_NAME),
//             deserializer: None,
//             serializer: None
//         }
//     }

//     fn deserialize(&mut self, callback: FD) {
//         self.deserializer = Some(callback);
//     }

//     fn serialize(&mut self, callback: FS) { 
//         self.serializer = Some(callback);
//     }

//     fn call(&self) -> Result<u64, String>  {
//         let mut input: Vec<u8> = vec![];
//         let output: Vec<u8> = Vec::with_capacity(1000);

//         let mut io_input: Option<Vec<u8>> = None;

//         if let Some(ref serializer) = self.serializer {
//             serializer(&mut input);
//             io_input = Some(input);
//         }


//         let mut cursor = self.device.call(self.control.clone().into(), io_input, Some(output)).expect("SentryChannel::call");

//         if let Some(ref deserializer) = self.deserializer {
//             return Ok(deserializer(&mut cursor))
//         }

//         Err("Deserializer wasn't used".to_string())
//     }
// }

// pub fn template() -> Result<u64, String> {
//     let mut channel = SentryChannel::new(0x0A00);

//     channel.serialize(|_| ());
//     channel.deserialize(|cursor| cursor.read_u64::<LittleEndian>().expect("IOCTL Buffer is wrong."));
//     channel.call()
// }
