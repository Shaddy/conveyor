// Copyright © ByteHeed.  All rights reserved.


use super::winapi::shared::minwindef::DWORD;

pub const SERVICE_KERNEL_DRIVER: DWORD = 0x0000_0001;
pub const SERVICE_DEMAND_START: DWORD  = 0x0000_0003;
pub const SERVICE_ERROR_NORMAL: DWORD  = 0x0000_0001;