// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{LPVOID, ULONG, LPHANDLE, USHORT};
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

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum FieldKey {
    PROCESS_ID,
    SID
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum MatchType { 
    EQUAL,
    GREATER,
    LESS,
    GREATER_OR_EQUAL,
    LESS_OR_EQUAL,
    NOT_EQUAL
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum ValueType { 
    EMPTY,
    UINT8,
    UINT16,
    UINT32,
    UINT64,
    SID_TYPE,
    UNICODE_STRING_TYPE
}

STRUCT!{
    #[derive(Debug)]
    struct MG_FIELD_VALUE   {
        Kind: ValueType,
        Value: u64,
    }
}

STRUCT!{
    #[derive(Debug)]
    struct MG_GUARD_CONDITION   {
        Field: FieldKey,
        Match: MatchType,
        Value: MG_FIELD_VALUE,
    }
}

impl RawStruct<MG_GUARD_CONDITION> for MG_GUARD_CONDITION { }



pub type LPMG_GUARD_CONDITION = *mut MG_GUARD_CONDITION;

STRUCT!{
    #[derive(Debug)]
    struct MG_GUARD_FILTER {
        NumberOfConditions: USHORT,
        Conditions: [MG_GUARD_CONDITION; 16],
    }
}


// impl MG_GUARD_FILTER {
//     pub unsafe fn from_raw(ptr: *const u8) -> MG_GUARD_FILTER {
//         mem::transmute(&*ptr)
//     }
// }

impl RawStruct<MG_GUARD_FILTER> for MG_GUARD_FILTER { }