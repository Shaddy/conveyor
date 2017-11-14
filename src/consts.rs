extern crate winapi;

use winapi::minwindef::DWORD;

pub const SERVICE_KERNEL_DRIVER: DWORD = 0x00000001;
pub const SERVICE_DEMAND_START: DWORD = 0x00000003;
pub const SERVICE_ERROR_NORMAL: DWORD = 0x00000001;