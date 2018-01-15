// Copyright © ByteHeed.  All rights reserved.

use super::winapi::um::winsvc::{SC_HANDLE};
use super::winapi::shared::ntdef::{PVOID, ULONG, PULONG, NTSTATUS};
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

#[repr(C)]
pub enum SystemInformationClass {
    SystemModuleInformationEx = 11,
}

#[link(name = "ntdll")]
extern "stdcall" {
    pub fn NtQuerySystemInformation(
        SystemInformationClass: SystemInformationClass,
        SystemInformation: PVOID,
        SystemInformationLength: ULONG,
        ReturnLength: PULONG
    ) -> NTSTATUS;
}