// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{LPHANDLE, LPVOID, ULONG, USHORT};
use super::winapi::shared::ntdef::HANDLE;
use std::mem;

type ULONG64 = u64;
type SIZE_T = usize;

pub trait RawStruct<T> {
    fn init() -> T {
        let s: T = unsafe { mem::zeroed() };
        s
    }

    fn size(&self) -> usize {
        mem::size_of::<T>()
    }

    fn as_ptr(&self) -> LPVOID {
        self as *const Self as LPVOID
    }

    fn as_mut_ptr(&mut self) -> LPVOID {
        self as *mut Self as LPVOID
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MapMode {
    KernelMode,
    UserMode,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_MAP_VIRTUAL_MEMORY {
    pub ToProcessId: ULONG64,
    pub MapMode: MapMode,
    pub BaseAddress: LPVOID,
    pub MapToAddress: LPVOID,
    pub Size: ULONG,
    pub Mdl: LPVOID,
    pub MappedMemory: LPVOID,
}

// pub type LPSE_MAP_VIRTUAL_MEMORY = *mut SE_MAP_VIRTUAL_MEMORY;

impl RawStruct<SE_MAP_VIRTUAL_MEMORY> for SE_MAP_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_UNMAP_VIRTUAL_MEMORY {
    pub Mdl: LPVOID,
    pub MappedMemory: LPVOID,
}

impl RawStruct<SE_UNMAP_VIRTUAL_MEMORY> for SE_UNMAP_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_READ_PROCESS_MEMORY {
    pub ProcessId: ULONG64,
    pub BaseAddress: LPVOID,
    pub Buffer: LPVOID,
    pub BytesToRead: SIZE_T,
    pub BytesCopied: SIZE_T,
}

impl RawStruct<SE_READ_PROCESS_MEMORY> for SE_READ_PROCESS_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_WRITE_PROCESS_MEMORY {
    pub ProcessId: ULONG64,
    pub BaseAddress: LPVOID,
    pub Buffer: LPVOID,
    pub BytesToWrite: SIZE_T,
    pub BytesCopied: SIZE_T,
}

impl RawStruct<SE_WRITE_PROCESS_MEMORY> for SE_WRITE_PROCESS_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_ALLOC_VIRTUAL_MEMORY {
    pub BaseAddress: LPVOID,
    pub Size: SIZE_T,
}

impl RawStruct<SE_ALLOC_VIRTUAL_MEMORY> for SE_ALLOC_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_FREE_VIRTUAL_MEMORY {
    pub BaseAddress: LPVOID,
}

impl RawStruct<SE_FREE_VIRTUAL_MEMORY> for SE_FREE_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_SECURE_VIRTUAL_MEMORY {
    pub BaseAddress: LPVOID,
    pub Size: SIZE_T,
    pub ProbeMode: ULONG,
    pub SecureHandle: LPHANDLE,
}

impl RawStruct<SE_SECURE_VIRTUAL_MEMORY> for SE_SECURE_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_UNSECURE_VIRTUAL_MEMORY {
    pub SecureHandle: LPHANDLE,
}

impl RawStruct<SE_UNSECURE_VIRTUAL_MEMORY> for SE_UNSECURE_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_COPY_VIRTUAL_MEMORY {
    pub ToAddress: LPVOID,
    pub FromAddress: LPVOID,
    pub Size: SIZE_T,
}

impl RawStruct<SE_COPY_VIRTUAL_MEMORY> for SE_COPY_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_READ_VIRTUAL_MEMORY {
    pub BaseAddress: LPVOID,
    pub Buffer: LPVOID,
    pub BytesToRead: ULONG,
    pub BytesCopied: ULONG,
}

impl RawStruct<SE_READ_VIRTUAL_MEMORY> for SE_READ_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_WRITE_VIRTUAL_MEMORY {
    pub BaseAddress: LPVOID,
    pub Buffer: LPVOID,
    pub BytesToWrite: ULONG,
    pub BytesCopied: ULONG,
}

impl RawStruct<SE_WRITE_VIRTUAL_MEMORY> for SE_WRITE_VIRTUAL_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_ALLOC_PROCESS_MEMORY {
    pub ProcessId: ULONG64,
    pub BaseAddress: LPVOID,
    pub BytesToAlloc: SIZE_T,
}

impl RawStruct<SE_ALLOC_PROCESS_MEMORY> for SE_ALLOC_PROCESS_MEMORY {}

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct SE_FREE_PROCESS_MEMORY {
    pub ProcessId: ULONG64,
    pub BaseAddress: LPVOID,
}

impl RawStruct<SE_FREE_PROCESS_MEMORY> for SE_FREE_PROCESS_MEMORY {}

pub enum TokenType {
    HijackSystem = 1,
    DuplicateSource,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SE_STEAL_TOKEN {
    pub SourcePid: ULONG64,
    pub TargetPid: ULONG64,
    pub StealType: u32,
}

impl RawStruct<SE_STEAL_TOKEN> for SE_STEAL_TOKEN {}

#[repr(C)]
#[derive(Clone)]
pub struct SE_GET_EXPORT_ADDRESS {
    pub ModuleBase: ULONG64,
    pub Name: [u8; 260],
    pub Address: ULONG64,
}

use std::fmt;

impl fmt::Debug for SE_GET_EXPORT_ADDRESS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self
            .Name
            .iter()
            .map(|&c| char::from(c))
            .take_while(|&c| c != char::from(00))
            .collect::<String>();

        write!(
            f,
            "SE_GET_EXPORT_ADDRESS {{ base: {:016x},
                                         name: {:?},
                                         addr: 0x{:016x} }}",
            self.ModuleBase, name, self.Address
        )
    }
}

impl RawStruct<SE_GET_EXPORT_ADDRESS> for SE_GET_EXPORT_ADDRESS {}

#[derive(Clone)]
pub struct RTL_PROCESS_MODULE_INFORMATION {
    pub Section: HANDLE,
    pub MappedBase: LPVOID,
    pub ImageBase: LPVOID,
    pub ImageSize: ULONG,
    pub Flags: ULONG,
    pub LoadOrderIndex: USHORT,
    pub InitOrderIndex: USHORT,
    pub LoadCount: USHORT,
    pub OffsetToFileName: USHORT,
    pub FullPathName: [u8; 256],
}

impl fmt::Debug for RTL_PROCESS_MODULE_INFORMATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self
            .FullPathName
            .iter()
            .map(|&c| char::from(c))
            .take_while(|&c| c != char::from(00))
            .collect::<String>();

        write!(
            f,
            "RTL_PROCESS_MODULE_INFO {{ section:     {:016x},
                                            base:          {:016x},
                                            image-base:    {:016x},
                                            image-size:    {:016x},
                                            flags:         {:016x},
                                            ..
                                            name:          {},
                                            }}",
            self.Section as u64,
            self.MappedBase as u64,
            self.ImageBase as u64,
            self.ImageSize,
            self.Flags,
            name
        )
    }
}

impl RawStruct<RTL_PROCESS_MODULE_INFORMATION> for RTL_PROCESS_MODULE_INFORMATION {}

#[repr(C)]
#[derive(Debug, Clone)]
struct RTL_PROCESS_MODULE_INFORMATION_EX {
    NextOffset: USHORT,
    BaseInfo: RTL_PROCESS_MODULE_INFORMATION,
    ImageChecksum: ULONG,
    TimeDateStamp: ULONG,
    DefaultBase: LPVOID,
}

impl RawStruct<RTL_PROCESS_MODULE_INFORMATION_EX> for RTL_PROCESS_MODULE_INFORMATION_EX {}

// typedef struct _RTL_PROCESS_MODULES
// {
//     ULONG NumberOfModules;
//     RTL_PROCESS_MODULE_INFORMATION Modules[1];
// } RTL_PROCESS_MODULES, *PRTL_PROCESS_MODULES;
