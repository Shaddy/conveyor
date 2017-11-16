// Copyright © ByteHeed.  All rights reserved.

use super::winapi::winsvc::{SC_HANDLE};
use super::winapi::minwindef::{BOOL, DWORD};
use super::winapi::winnt::{LPCWSTR};

#[link(name = "advapi32")]
extern "stdcall" {
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: LPCWSTR
    ) -> BOOL;
}