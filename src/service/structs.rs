// Copyright © ByteHeed.  All rights reserved.
#![allow(non_camel_case_types, non_snake_case, dead_code)]

use super::winapi::shared::minwindef::{DWORD};


STRUCT!{
    #[derive(Debug)]
    struct SERVICE_STATUS_PROCESS  {
        dwServiceType: DWORD,
        dwCurrentState: DWORD,
        dwControlsAccepted: DWORD,
        dwWin32ExitCode: DWORD,
        dwServiceSpecificExitCode: DWORD,
        dwCheckPoint: DWORD,
        dwWaitHint: DWORD,
        dwProcessId: DWORD,
        dwServiceFlags: DWORD,
}}

pub type LPSERVICE_STATUS_PROCESS = *mut SERVICE_STATUS_PROCESS;

// SERVICE_STATUS_PROCESS wrapper in a rusty way.
//
// Pending:
//
// - Add enumerators for dwControlsAccepted.
// - Add proper 'system' discriminator
//   ( Could exists a case where service is system but is not running, so flags would be 0 )
// - Add proper timing dwWaitHint estimation.

#[derive(Debug, PartialEq)]
pub struct ServiceInfo {
    pub kind: ServiceType,
    pub status: ServiceStatus,
    pub pid: u32,
    pub system: bool
}

impl From<SERVICE_STATUS_PROCESS> for ServiceInfo {
    fn from(info: SERVICE_STATUS_PROCESS) -> Self {
        ServiceInfo {
            kind: ServiceType::from_bits(info.dwServiceType)
            .expect("Unable to parse dwServiceType"),
            status: ServiceStatus::from(info.dwCurrentState),
            pid: info.dwProcessId,
            system: info.dwServiceFlags == 1
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServiceStatus {
    ServiceRunning,
    PausePending,
    ContinuePending,
    Paused,
    Running,
    StartPending,
    StopPending,
    Stopped
}

bitflags! {
    pub struct ServiceType: u32 {
        const FILE_SYSTEM_DRIVER       = 0x0000_0001;
        const KERNEL_DRIVER            = 0x0000_0002;
        const WIN32_OWN_PROCESS        = 0x0000_0010;
        const WIN32_SHARE_PROCESS      = 0x0000_0020;
    }
}

impl From<u32> for ServiceStatus {
    fn from(value: u32) -> Self {
        match value {
            1 => ServiceStatus::Stopped,
            2 => ServiceStatus::StartPending,
            3 => ServiceStatus::StopPending,
            4 => ServiceStatus::Running,
            5 => ServiceStatus::ContinuePending,
            6 => ServiceStatus::PausePending,
            7 => ServiceStatus::Paused,
            _ => panic!("Unable to convert value: {} to ServiceStatus", value)
        }

    }
}