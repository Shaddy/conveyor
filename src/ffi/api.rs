// Copyright © ByteHeed.  All rights reserved.

use super::winapi::um::winsvc::{SC_HANDLE};
use super::winapi::shared::minwindef::{BOOL, DWORD};
use super::winapi::um::winnt::{LPCWSTR};

#[link(name = "advapi32")]
extern "stdcall" {
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: LPCWSTR
    ) -> BOOL;
}