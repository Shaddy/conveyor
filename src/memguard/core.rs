// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::iochannel::{ Device, IoCtl };
use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };
use super::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use super::num::FromPrimitive;

use std::ops::Fn;
use std::io::Cursor;

const IOCTL_SENTRY_TYPE: u32 = 0xB080;

const SE_NT_DEVICE_NAME: &'static str = "\\\\.\\Sentry";

enum_from_primitive! {
    #[derive(Debug, Clone)]
    enum PartitionOption {
        TraceDebugEvents = 1,
        TraceToDisk,
        CoalesceNotifications,
        CollectStats,
        SecureMode,
    }
}

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

pub fn get_partition_option(id: u64, option: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    
    println!("getting-info - id: {} | option: {} ({:?})", id, option, PartitionOption::from_u64(option).unwrap());
    println!("input: {:?}", input);
    let mut cursor = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_GETOPTION_PARTITION");

    let option_value = cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong");

    println!("id: {} | option: {} | value: {} ", id, option, option_value);
}

pub fn set_partition_option(id: u64, option: u64, value: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A03, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    input.write_u64::<LittleEndian>(value).unwrap();
    
    println!("{:?}", PartitionOption::from_u64(option));
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_GETOPTION_PARTITION");

    println!("id: {} | option: {:?} | value: {} ", id, option, value);
}

pub fn register_guard(id: u64, option: u64, value: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A10, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    input.write_u64::<LittleEndian>(value).unwrap();
    
    println!("{:?}", PartitionOption::from_u64(option));
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_GETOPTION_PARTITION");

    println!("id: {} | option: {:?} | value: {} ", id, option, value);
}

pub fn unregister_guard(_id: u64) {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A11, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn control_guard(_id: u64) {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A12, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn create_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A20, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn delete_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A21, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn add_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A22, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn remove_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A23, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn set_state_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A24, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn get_info_region() {
    let _control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A25, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS );
    unimplemented!()
}

pub fn enumerate_region() {
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