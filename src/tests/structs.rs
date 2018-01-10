// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{LPVOID};
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


#[derive(Debug, Copy, Clone)]
pub enum TestType {
    BasicGuard = 1,
    BasicGuardedRegion,
    BasicTracePoint,
    BasicIntercept,
    DelayIntercept,
    PageFaultIntercept,
    PriorityIntercept,
    TimerIntercept
}

bitflags! {
    pub struct TestFlags: u16 {
        const INTERCEPT_NORMAL          = 0x00000000;
        const INTERCEPT_STRESS_AFFINITY = 0x00000001;
    }
}

STRUCT!{
    #[derive(Debug)]
    struct SE_RUN_TEST {
        Kind: TestType,
        Flags: TestFlags,
    }
}

impl RawStruct<SE_RUN_TEST> for SE_RUN_TEST { }