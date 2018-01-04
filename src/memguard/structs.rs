// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{LPVOID, ULONG, LPHANDLE};
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


STRUCT!{
    #[derive(Debug)]
    struct SE_MAP_VIRTUAL_MEMORY  {
        ToProcessId: ULONG64,
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

STRUCT!{
    #[derive(Debug)]
    struct SE_GET_CURRENT_EPROCESS   {
        Process: ULONG64,
}}

impl RawStruct<SE_GET_CURRENT_EPROCESS> for SE_GET_CURRENT_EPROCESS { }

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