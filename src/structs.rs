#![allow(non_camel_case_types, non_snake_case, dead_code)]

use winapi::minwindef::{DWORD};


STRUCT!{
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