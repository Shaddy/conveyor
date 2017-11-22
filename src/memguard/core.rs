// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::iochannel::{ Device, IoCtl };
use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };
use super::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

const IOCTL_SENTRY_TYPE: u32 = 0xB080;

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


const SE_NT_DEVICE_NAME: &'static str = "\\\\.\\Sentry";

#[derive(Debug, Clone)]
enum PartitionOption {
    TraceDebugEvents = 1,
    TraceToDisk,
    CoalesceNotifications,
    CollectStats,
    SecureMode,
}

#[derive(Debug)]
pub struct Partition {
    id: u64
}


pub fn create_partition() -> Result<Partition, String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A00, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let input = Vec::with_capacity(1000);
    let output: Vec<u8> = Vec::with_capacity(1000);

    
    let mut cursor = Device::new(SE_NT_DEVICE_NAME)
                            .call(control.into(), Some(input), Some(output))
                            .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");

    Ok(Partition { id: cursor.read_u64::<LittleEndian>().expect("IOCTL Buffer is wrong.")})
}

pub fn delete_partition(id: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A01, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];

    input.write_u64::<LittleEndian>(id).expect("delete_partition() - Failed to write partition id into buffer");
    
    let _ = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(vec![]))
                .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");

}

pub fn get_partition_option(id: u64, option: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    
    let cursor = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");

    let option_value = cursor.read_u64::<LittleEndian>().expect("get_partition_option() - IOCTL Buffer is wrong");

    println!("id: {} | option: {} | value: {} ", id, option, option_value);
}

pub fn set_partition_option(id: u64, option: u64, value: u64) {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A02, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    let mut input = vec![];
    let output: Vec<u8> = Vec::with_capacity(1000);

    input.write_u64::<LittleEndian>(id).unwrap();
    input.write_u64::<LittleEndian>(option).unwrap();
    input.write_u64::<LittleEndian>(value).unwrap();
    
    let cursor = Device::new(SE_NT_DEVICE_NAME)
                .call(control.into(), Some(input), Some(output))
                .expect("Error calling IOCTL_SENTRY_CREATE_PARTITION");

    let option = PartitionOption::from_int(1);
    println!("id: {} | option: {:?} | value: {} ", id, option, value);
}