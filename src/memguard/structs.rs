// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::minwindef::{DWORD, LPVOID, ULONG};
use std::mem;

type ULONG64 = u64;
type SIZE_T = usize;

STRUCT!{
    #[derive(Debug)]
    struct SE_MAP_VIRTUAL_MEMORY  {
        ProcessId: LPVOID,
        BaseAddress: LPVOID,
        MapToAddress: LPVOID,
        Size: ULONG,
        Mdl: LPVOID,
        MappedMemory: LPVOID,
}}

// pub type LPSE_MAP_VIRTUAL_MEMORY = *mut SE_MAP_VIRTUAL_MEMORY;

impl SE_MAP_VIRTUAL_MEMORY {
    pub fn init() -> SE_MAP_VIRTUAL_MEMORY {
        let s: SE_MAP_VIRTUAL_MEMORY = unsafe { mem::zeroed() };
        s
    }

    pub fn as_ptr(&self) -> LPVOID {
        unsafe { mem::transmute::<&SE_MAP_VIRTUAL_MEMORY, LPVOID>(self) }
    }

    pub fn as_mut_ptr(&mut self) -> LPVOID {
        unsafe { mem::transmute::<&mut SE_MAP_VIRTUAL_MEMORY, LPVOID>(self) }
    }

    pub fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

STRUCT!{
    #[derive(Debug)]
    struct SE_UNMAP_VIRTUAL_MEMORY  {
        Mdl: LPVOID,
        MappedMemory: LPVOID,
}}

impl SE_UNMAP_VIRTUAL_MEMORY {
    pub fn init() -> SE_UNMAP_VIRTUAL_MEMORY {
        let s: SE_UNMAP_VIRTUAL_MEMORY = unsafe { mem::zeroed() };
        s
    }

    pub fn as_ptr(&self) -> LPVOID {
        unsafe { mem::transmute::<&SE_UNMAP_VIRTUAL_MEMORY, LPVOID>(self) }
    }

    pub fn as_mut_ptr(&mut self) -> LPVOID {
        unsafe { mem::transmute::<&mut SE_UNMAP_VIRTUAL_MEMORY, LPVOID>(self) }
    }

    pub fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}


pub trait RawStruct<T> {
    fn init() -> T {
        let s: T = unsafe { mem::zeroed() };
        s
    }

    fn size(&self) -> usize {
        mem::size_of::<T>()
    }

    fn as_ptr(&self) -> LPVOID;
    fn as_mut_ptr(&mut self) -> LPVOID;

}

STRUCT!{
    #[derive(Debug)]
    struct SE_READ_PROCESS_MEMORY   {
        ProcessId: ULONG64,
        BaseAddress: LPVOID,
        Buffer: LPVOID,
        BytesToRead: SIZE_T,
        BytesCopied: SIZE_T,
}}

impl RawStruct<SE_READ_PROCESS_MEMORY> for SE_READ_PROCESS_MEMORY {
    fn as_ptr(&self) -> LPVOID {
        unsafe { mem::transmute::<&Self, LPVOID>(self) }
    }

    fn as_mut_ptr(&mut self) -> LPVOID {
        unsafe { mem::transmute::<&mut Self, LPVOID>(self) }
    }
}

STRUCT!{
    #[derive(Debug)]
    struct SE_WRITE_PROCESS_MEMORY   {
        ProcessId: ULONG64,
        BaseAddress: LPVOID,
        Buffer: LPVOID,
        BytesToWrite: SIZE_T,
        BytesCopied: SIZE_T,
}}

impl RawStruct<SE_WRITE_PROCESS_MEMORY> for SE_WRITE_PROCESS_MEMORY {
    fn as_ptr(&self) -> LPVOID {
        unsafe { mem::transmute::<&Self, LPVOID>(self) }
    }

    fn as_mut_ptr(&mut self) -> LPVOID {
        unsafe { mem::transmute::<&mut Self, LPVOID>(self) }
    }
}
