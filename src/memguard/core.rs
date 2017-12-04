// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::iochannel::{ Device, IoCtl };
use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };
use super::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use super::num::FromPrimitive;
use super::{Access, Range, Status};


const IOCTL_SENTRY_TYPE: u32 = 0xB080;

const SE_NT_DEVICE_NAME: &'static str = "\\\\.\\Sentry";

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

// use std::ops::Fn;
// struct SentryChannel<FS, FD> {
//     control: IoCtl,
//     device: Device,
//     deserializer: Option<FD>,
//     serializer: Option<FS>
// }

// impl<FS, FD> SentryChannel<FS, FD>
//     where FD: Fn(&mut Cursor<Vec<u8>>) -> u64,
//           FS: Fn(&mut Vec<u8>) { 
//     fn new(function: usize) -> Self {
//         SentryChannel {
//             control: IoCtl::new(IOCTL_SENTRY_TYPE, function as u32, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS),
//             device: Device::new(SE_NT_DEVICE_NAME),
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


pub fn create_partition() -> Result<u64, String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A00, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let input = Vec::with_capacity(1000);
    let output: Vec<u8> = Vec::with_capacity(1000);

    
    let mut cursor = Device::new(SE_NT_DEVICE_NAME)
                            .call(control.into(), Some(input), Some(output))
                            .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");

    Ok( cursor.read_u64::<LittleEndian>().expect("IOCTL Buffer is wrong.") )
}

pub fn delete_partition(id: u64) -> Result<(), String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A01, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).expect("delete_partition() - Failed to write partition id into buffer");
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(vec![]))
                .expect("Error calling IOCTL_SENTRY_DELETE_PARTITION");

    Ok(())

}

#[derive(Debug)]
pub enum PartitionError {
    NotExists,
    UnknownError
}

pub fn get_partition_option(id: u64, option: u64) -> Result<u64, PartitionError> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    
    let mut cursor = match Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output)) 
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

pub fn _set_partition_option(id: u64, option: u64, value: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A03, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    input.write_u64::<LittleEndian>(value).unwrap();
    
    println!("{:?}", PartitionOption::from_u64(option));
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_SETOPTION_PARTITION");

    println!("id: {} | option: {:?} | value: {} ", id, option, value);
}

pub fn register_guard_extended(id: u64, context: u64, filter: u64, flags: u64, priority: u64, _function: u64) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A10, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(context).unwrap();
    input.write_u64::<LittleEndian>(filter).unwrap();
    input.write_u64::<LittleEndian>(flags).unwrap();
    input.write_u64::<LittleEndian>(priority).unwrap();
    
    let mut cursor = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_REGISTER_GUARD");


    cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong")
}

pub fn register_guard(id: u64) -> Result<u64, String> {
    Ok(register_guard_extended(id, 0, 0, 0, 0, 0))
}

pub fn unregister_guard(id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A11, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).unwrap();
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), None)
                .expect("Error unregistering guard");
}

enum Control {
    Start = 0,
    Stop
}

pub fn stop_guard(id: u64) {
    control_guard(id, Control::Stop)
}

pub fn start_guard(id: u64) {
    control_guard(id, Control::Start)
}

fn control_guard(id: u64, action: Control) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A12, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(action as u64).unwrap();
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), None)
                .expect("control_guard()");
}

pub fn create_region(partition_id: u64, range: &Range, access: Access, weight: Option<usize>) -> u64 {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A20, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    let mut input = vec![];

    input.write_u64::<LittleEndian>(partition_id).unwrap();
    input.write_u64::<LittleEndian>(range.base).unwrap();
    input.write_u64::<LittleEndian>(range.limit).unwrap();

    // each regions starts disabled
    input.write_u32::<LittleEndian>(Status::ENABLED.bits()).unwrap(); // flags

    // access
    input.write_u32::<LittleEndian>(access.bits()).unwrap();

    // action
    input.write_u64::<LittleEndian>(0).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();
    input.write_u64::<LittleEndian>(0).unwrap();

    input.write_u64::<LittleEndian>(weight.unwrap_or(0) as u64).unwrap();

    let output: Vec<u8> = Vec::with_capacity(1000);
    let mut cursor = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("create_region()");

    cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong")
}

pub fn delete_region(region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A21, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), None)
                .expect("delete_region()");
}

pub fn add_region(guard_id: u64, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A22, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), None)
                .expect("add_region()");
}

pub fn remove_region(guard_id: u64, region_id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A23, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];

    input.write_u64::<LittleEndian>(guard_id).unwrap();
    input.write_u64::<LittleEndian>(region_id).unwrap();
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), None)
                .expect("remove_region()");
}

pub fn _set_state_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A24, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn _get_info_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A25, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn _enumerate_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A26, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

// modify IOCTL to be a constant
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