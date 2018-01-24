// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{LPVOID, ULONG, LPHANDLE, USHORT};
use super::winapi::shared::ntdef::{HANDLE};
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
    UserMode
}

STRUCT!{
    #[derive(Debug)]
    struct SE_MAP_VIRTUAL_MEMORY  {
        ToProcessId: ULONG64,
        MapMode: MapMode,
        BaseAddress: LPVOID,
        MapToAddress: LPVOID,
        Size: ULONG,
        Mdl: LPVOID,
        MappedMemory: LPVOID,
}}

// pub type LPSE_MAP_VIRTUAL_MEMORY = *mut SE_MAP_VIRTUAL_MEMORY;

impl RawStruct<SE_MAP_VIRTUAL_MEMORY> for SE_MAP_VIRTUAL_MEMORY  { }

STRUCT!{
    #[derive(Debug)]
    struct SE_UNMAP_VIRTUAL_MEMORY  {
        Mdl: LPVOID,
        MappedMemory: LPVOID,
}}

impl RawStruct<SE_UNMAP_VIRTUAL_MEMORY> for SE_UNMAP_VIRTUAL_MEMORY  { }

STRUCT!{
    #[derive(Debug)]
    struct SE_READ_PROCESS_MEMORY   {
        ProcessId: ULONG64,
        BaseAddress: LPVOID,
        Buffer: LPVOID,
        BytesToRead: SIZE_T,
        BytesCopied: SIZE_T,
}}

impl RawStruct<SE_READ_PROCESS_MEMORY> for SE_READ_PROCESS_MEMORY { }

STRUCT!{
    #[derive(Debug)]
    struct SE_WRITE_PROCESS_MEMORY   {
        ProcessId: ULONG64,
        BaseAddress: LPVOID,
        Buffer: LPVOID,
        BytesToWrite: SIZE_T,
        BytesCopied: SIZE_T,
}}

impl RawStruct<SE_WRITE_PROCESS_MEMORY> for SE_WRITE_PROCESS_MEMORY { }

STRUCT!{
    #[derive(Debug)]
    struct SE_ALLOC_VIRTUAL_MEMORY   {
        BaseAddress: LPVOID,
        Size: SIZE_T,
}}

impl RawStruct<SE_ALLOC_VIRTUAL_MEMORY> for SE_ALLOC_VIRTUAL_MEMORY { }

STRUCT!{
    #[derive(Debug)]
    struct SE_FREE_VIRTUAL_MEMORY   {
        BaseAddress: LPVOID,
}}

impl RawStruct<SE_FREE_VIRTUAL_MEMORY> for SE_FREE_VIRTUAL_MEMORY { }


STRUCT!{
    #[derive(Debug)]
    struct SE_SECURE_VIRTUAL_MEMORY   {
        BaseAddress: LPVOID,
        Size: SIZE_T,
        ProbeMode: ULONG,
        SecureHandle: LPHANDLE,
}}

impl RawStruct<SE_SECURE_VIRTUAL_MEMORY> for SE_SECURE_VIRTUAL_MEMORY { }


STRUCT!{
    #[derive(Debug)]
    struct SE_UNSECURE_VIRTUAL_MEMORY   {
        SecureHandle: LPHANDLE,
}}

impl RawStruct<SE_UNSECURE_VIRTUAL_MEMORY> for SE_UNSECURE_VIRTUAL_MEMORY { }

STRUCT!{
    #[derive(Debug)]
    struct SE_COPY_VIRTUAL_MEMORY   {
        ToAddress: LPVOID,
        FromAddress: LPVOID,
        Size: SIZE_T,
}}

impl RawStruct<SE_COPY_VIRTUAL_MEMORY> for SE_COPY_VIRTUAL_MEMORY { }


STRUCT!{
    #[derive(Debug)]
    struct SE_READ_VIRTUAL_MEMORY   {
        BaseAddress: LPVOID,
        Buffer: LPVOID,
        BytesToRead: ULONG,
        BytesCopied: ULONG,
}}

impl RawStruct<SE_READ_VIRTUAL_MEMORY> for SE_READ_VIRTUAL_MEMORY { }


STRUCT!{
    #[derive(Debug)]
    struct SE_WRITE_VIRTUAL_MEMORY   {
        BaseAddress: LPVOID,
        Buffer: LPVOID,
        BytesToWrite: ULONG,
        BytesCopied: ULONG,
}}

impl RawStruct<SE_WRITE_VIRTUAL_MEMORY> for SE_WRITE_VIRTUAL_MEMORY { }


STRUCT!{
    #[derive(Debug)]
    struct SE_ALLOC_PROCESS_MEMORY   {
        ProcessId: ULONG64,
        BaseAddress: LPVOID,
        BytesToAlloc: SIZE_T,
}}

impl RawStruct<SE_ALLOC_PROCESS_MEMORY> for SE_ALLOC_PROCESS_MEMORY { }


STRUCT!{
    #[derive(Debug)]
    struct SE_FREE_PROCESS_MEMORY   {
        ProcessId: ULONG64,
        BaseAddress: LPVOID,
}}

impl RawStruct<SE_FREE_PROCESS_MEMORY> for SE_FREE_PROCESS_MEMORY { }

pub enum TokenType {
    HijackSystem = 1,
    DuplicateSource
}

STRUCT!{
    #[derive(Debug)]
    struct SE_STEAL_TOKEN   {
        SourcePid: ULONG64,
        TargetPid: ULONG64,
        StealType: u32,
}}

impl RawStruct<SE_STEAL_TOKEN> for SE_STEAL_TOKEN { }

STRUCT!{
    struct SE_GET_EXPORT_ADDRESS {
        ModuleBase: ULONG64,
        Name: [u8; 260],
        Address: ULONG64,
    }
}

use std::fmt;

impl fmt::Debug for SE_GET_EXPORT_ADDRESS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self.Name.iter()
                    .map(|&c| char::from(c))
                    .take_while(|&c| c != char::from(00)).collect::<String>();

        write!(f, "SE_GET_EXPORT_ADDRESS {{ base: {:016x},
                                         name: {:?},
                                         addr: 0x{:016x} }}",
        self.ModuleBase,
        name,
        self.Address)
    }
}

impl RawStruct<SE_GET_EXPORT_ADDRESS> for SE_GET_EXPORT_ADDRESS { }

STRUCT!{
    struct RTL_PROCESS_MODULE_INFORMATION {
        Section: HANDLE,
        MappedBase: LPVOID,
        ImageBase: LPVOID,
        ImageSize: ULONG,
        Flags: ULONG,
        LoadOrderIndex: USHORT,
        InitOrderIndex: USHORT,
        LoadCount: USHORT,
        OffsetToFileName: USHORT,
        FullPathName: [u8; 256],
    }
}

impl fmt::Debug for RTL_PROCESS_MODULE_INFORMATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self.FullPathName.iter()
                    .map(|&c| char::from(c))
                    .take_while(|&c| c != char::from(00)).collect::<String>();

        write!(f, "RTL_PROCESS_MODULE_INFO {{ section:     {:016x},
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
                                name)
    }

}

impl RawStruct<RTL_PROCESS_MODULE_INFORMATION> for RTL_PROCESS_MODULE_INFORMATION { }

STRUCT!{
    struct RTL_PROCESS_MODULE_INFORMATION_EX {

        NextOffset: USHORT,
        BaseInfo: RTL_PROCESS_MODULE_INFORMATION,
        ImageChecksum: ULONG,
        TimeDateStamp: ULONG,
        DefaultBase: LPVOID,
    }
}

impl RawStruct<RTL_PROCESS_MODULE_INFORMATION_EX> for RTL_PROCESS_MODULE_INFORMATION_EX { }


// typedef struct _RTL_PROCESS_MODULES
// {
//     ULONG NumberOfModules;
//     RTL_PROCESS_MODULE_INFORMATION Modules[1];
// } RTL_PROCESS_MODULES, *PRTL_PROCESS_MODULES;
