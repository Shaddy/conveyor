// Copyright © ByteHeed.  All rights reserved.

// use ffi::traits::EncodeUtf16;

use super::iochannel::{ Device, IoCtl };
use super::winapi::{ FILE_READ_ACCESS, FILE_WRITE_ACCESS, METHOD_BUFFERED };

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



pub fn create_partition() -> Result<u32, String> {
    let control: IoCtl = IoCtl::new(IOCTL_SENTRY_TYPE, 0x0A00, METHOD_BUFFERED, FILE_READ_ACCESS | FILE_WRITE_ACCESS);

    if Device::new("\\\\.\\Sentry").call(control.into()).expect("Error calling IOCTL_SENTRY_CREATE_PARTITION") {
        return Ok(1);
    }

    Ok(0)
}

// #define SE_NT_DEVICE_NAME     L"\\Device\\Sentry"
// #define SE_WIN32_DEVICE_NAME  L"\\DosDevices\\Sentry"