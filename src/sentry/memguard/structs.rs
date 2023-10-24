// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{LPVOID, PULONG, ULONG, USHORT};
use super::winapi::shared::ntdef::{BOOLEAN, HANDLE};

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
    SID,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum MatchType {
    EQUAL,
    GREATER,
    LESS,
    GREATER_OR_EQUAL,
    LESS_OR_EQUAL,
    NOT_EQUAL,
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
    UNICODE_STRING_TYPE,
}

#[derive(Debug, Clone)]
pub(crate) struct MG_FIELD_VALUE {
    pub Kind: ValueType,
    pub Value: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct MG_GUARD_CONDITION {
    pub Field: FieldKey,
    pub Match: MatchType,
    pub Value: MG_FIELD_VALUE,
}

impl RawStruct<MG_GUARD_CONDITION> for MG_GUARD_CONDITION {}

pub type LPMG_GUARD_CONDITION = *mut MG_GUARD_CONDITION;

#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct MG_GUARD_FILTER {
    pub NumberOfConditions: USHORT,
    pub Conditions: [MG_GUARD_CONDITION; 16],
}

// impl MG_GUARD_FILTER {
//     pub unsafe fn from_raw(ptr: *const u8) -> MG_GUARD_FILTER {
//         mem::transmute(&*ptr)
//     }
// }

impl RawStruct<MG_GUARD_FILTER> for MG_GUARD_FILTER {}

pub enum ObjectType {
    OpenMessage,
    CloseMessage,
    DeleteMessage,
    ParseMessage,
    SecurityMessage,
    QueryNameMessage,
    OkayToCloseMessage,
}

type ULONG_PTR = usize;
type PVOID = LPVOID;
type OB_OPEN_REASON = u32;
type ACCESS_MASK = u32;
type KPROCESSOR_MODE = u32;
type SECURITY_OPERATION_CODE = u32;
type POOL_TYPE = u32;
type PGENERIC_MAPPING = PVOID;
type PEPROCESS = PVOID;
type PSECURITY_QUALITY_OF_SERVICE = PVOID;
type POBJECT_NAME_INFORMATION = PVOID;
type PACCESS_STATE = PVOID;
type PUNICODE_STRING = PVOID;
type PSECURITY_INFORMATION = PVOID;
type PSECURITY_DESCRIPTOR = PVOID;

#[derive(Debug)]
pub(crate) struct OPEN_MESSAGE {
    OpenReason: OB_OPEN_REASON,
    Process: PEPROCESS,
    Object: PVOID,
    GrantedAccess: ACCESS_MASK,
    HandleCount: ULONG,
}
impl RawStruct<OPEN_MESSAGE> for OPEN_MESSAGE {}

#[derive(Debug)]
pub(crate) struct CLOSE_MESSAGE {
    Process: PEPROCESS,
    Object: PVOID,
    GrantedAccess: ACCESS_MASK,
    ProcessHandleCount: ULONG_PTR,
    SystemHandleCount: ULONG_PTR,
}

impl RawStruct<CLOSE_MESSAGE> for CLOSE_MESSAGE {}

#[derive(Debug)]
pub(crate) struct DELETE_MESSAGE {
    Object: PVOID,
}

impl RawStruct<DELETE_MESSAGE> for DELETE_MESSAGE {}

#[derive(Debug)]
pub(crate) struct PARSE_MESSAGE {
    ParseObject: PVOID,
    ObjectType: PVOID,
    AccessState: PACCESS_STATE,
    AccessMode: KPROCESSOR_MODE,
    Attributes: ULONG,
    CompleteName: PUNICODE_STRING,
    RemainingName: PUNICODE_STRING,
    Context: PVOID,
    SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    Object: PVOID,
}
impl RawStruct<PARSE_MESSAGE> for PARSE_MESSAGE {}

#[derive(Debug)]
pub(crate) struct SECURITY_MESSAGE {
    Object: PVOID,
    OperationCode: SECURITY_OPERATION_CODE,
    SecurityInformation: PSECURITY_INFORMATION,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    CapturedLength: PULONG,
    ObjectsSecurityDescriptor: PSECURITY_DESCRIPTOR,
    PoolType: POOL_TYPE,
    GenericMapping: PGENERIC_MAPPING,
    PreviousMode: KPROCESSOR_MODE,
}

impl RawStruct<SECURITY_MESSAGE> for SECURITY_MESSAGE {}

#[derive(Debug)]
pub(crate) struct QUERYNAME_MESSAGE {
    Object: PVOID,
    HasObjectName: BOOLEAN,
    ObjectNameInfo: POBJECT_NAME_INFORMATION,
    Length: ULONG,
    ReturnLength: PULONG,
    PreviousMode: KPROCESSOR_MODE,
}

impl RawStruct<QUERYNAME_MESSAGE> for QUERYNAME_MESSAGE {}

#[derive(Debug)]
pub(crate) struct OKAYTOCLOSE_MESSAGE {
    Process: PEPROCESS,
    Object: PVOID,
    Handle: HANDLE,
    PreviousMode: KPROCESSOR_MODE,
}
impl RawStruct<OKAYTOCLOSE_MESSAGE> for OKAYTOCLOSE_MESSAGE {}
