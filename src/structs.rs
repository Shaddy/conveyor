#![allow(non_camel_case_types, non_snake_case, dead_code)]

use winapi::minwindef::{DWORD};
use std::num::FromPrimitive;


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


#[derive(FromPrimitive)]
enum ServiceType {
    FileSystemDriver,
    KernelDriver,
    Win32OwnProcess,
    Win32ShareProcess
}

enum ServiceStatus {
    ServiceRunning,
    PausePending,
    ContinuePending,
    Paused,
    Running,
    StartPending,
    StopPending,
    Stopped
}

impl SERVICE_STATUS_PROCESS {
    fn status(&self) -> ServiceStatus {
        match self.dwCurrentState {
            1 => ServiceStatus::Stopped,
            2 => ServiceStatus::StartPending,
            3 => ServiceStatus::StopPending,
            4 => ServiceStatus::Running,
            5 => ServiceStatus::ContinuePending,
            6 => ServiceStatus::PausePending,
            7 => ServiceStatus::Paused,
            _ => panic!("ServiceStatus can't convert {}", self.dwCurrentState)
        }
    }

    fn service_type(&self) -> ServiceType {
        match self.dwServiceType {
            1 => ServiceType::KernelDriver,
            2 => ServiceType::FileSystemDriver,
         0x10 => ServiceType::Win32OwnProcess,
         0x20 => ServiceType::Win32ShareProcess,
         _ => panic!("Dude, this is NOT a service type...")
        }
    }
}

