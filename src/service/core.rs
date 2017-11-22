// Copyright © ByteHeed.  All rights reserved.

// external namespaces

use std::ptr::null_mut;
use std::io::Error;
use std::mem::{transmute, zeroed, size_of_val};

use super::ffi;
use super::winapi;
use super::winapi::winsvc;
use super::winapi::winsvc::{SC_HANDLE};
use super::advapi32;

// custom namespaces

use super::ffi::traits::EncodeUtf16;
use super::structs::{SERVICE_STATUS_PROCESS, ServiceInfo};
use super::consts::{SERVICE_KERNEL_DRIVER, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL};


#[derive(Debug, PartialEq)]
pub enum ServiceError {
    DeletePending,
    ServiceDisabled,
    ServiceAlreadyExists,
    ServiceCannotAcceptCtrl,
    ServiceAlreadyRunning,
    ServiceDoesNotExist,
    AccessViolation,
    ServiceNotActive,
    InvalidHandle,
    UnknownError(i32),
}

#[derive(Debug, Clone)]
pub struct WindowsServiceControlManager {
    handle: SC_HANDLE
}

impl WindowsServiceControlManager {
    fn open() -> Result<SC_HANDLE, String> {
        let handle = unsafe {
            advapi32::OpenSCManagerW(
                null_mut(),
                null_mut(),
                winapi::winsvc::SC_MANAGER_ALL_ACCESS,
            )
        };

        if handle.is_null() {
            return Err(Error::last_os_error().to_string());
        }

        Ok(handle)
    }

    pub fn new() -> WindowsServiceControlManager {
        let handle = WindowsServiceControlManager::open().expect("Unable to open Service Control Manager");

        WindowsServiceControlManager {
            handle: handle
        }
    }
}

impl Drop for WindowsServiceControlManager {
    fn drop(&mut self) {
        WindowsService::close_service_handle(self.handle);
    }
}

#[derive(Debug, Clone)]
pub struct WindowsService {
    name: String,
    path: String,
    manager: WindowsServiceControlManager
}

impl WindowsService {
    pub fn new(name: &str, path: &str) -> WindowsService {
        let manager = WindowsServiceControlManager::new();

        WindowsService {
            name: name.to_string(),
            path: path.to_string(),
            manager: manager,
        }
    }

    pub fn remove(&self) -> Self {
        self.delete().expect("Can't remove service");
        println!("Service {:?} has been successfully removed", self.name);

        self.clone()
    }

    pub fn install(&self) -> Self {
        if let Err(err) = self.create() {
            match err {
                ServiceError::ServiceAlreadyExists => {
                    println!("Failed to install {:?}: Service already exists.", self.name);
                }
                _ => println!("Failed to install {:?}: unknown error {:?}", self.name, err),
            }
        } else {
            println!("Service {:?} has been successfully installed", self.name);
        }

        self.clone()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
    
    pub fn stop(&self) -> Self {
        let service = self.open().expect("Unable to open service");

        let mut status: winapi::SERVICE_STATUS = unsafe {zeroed()};

        let success = unsafe {
            advapi32::ControlService( 
            service, 
            winsvc::SERVICE_CONTROL_STOP, 
            &mut status) == 0
        };

        if success {
            println!("Can't stop service ({}): {:?}", self.name, WindowsService::service_error())
        }

        self.clone()
    }

    pub fn start(&self) -> Self {
        let service = self.open().expect("Unable to open service");

        let success = unsafe {
            ffi::StartServiceW(
                service,
                0,
                null_mut(),
            ) == 0
        };

        if success {
            println!("Can't start service ({}): {:?}", self.name, WindowsService::service_error())
        }

        self.clone()
    }


    fn query_service_status(
        service: SC_HANDLE
    ) -> Result<SERVICE_STATUS_PROCESS, String> {

        let mut process: SERVICE_STATUS_PROCESS = unsafe { zeroed() };
        let mut size: u32 = size_of_val(&process) as u32;

        let result = unsafe {
            advapi32::QueryServiceStatusEx(
                service,
                winsvc::SC_STATUS_PROCESS_INFO,
                transmute::<&mut SERVICE_STATUS_PROCESS, *mut u8>(&mut process),
                size,
                &mut size,
            )
        };

        match result == 0 {
            false => return Ok(process),
            true => return Err(Error::last_os_error().to_string())
        }
    }


    pub fn query(&self) -> ServiceInfo {
        let service = self.open().expect("Unable to open service");

        let info = WindowsService::query_service_status( service ).expect("Can't query service");

        ServiceInfo::from(info)
    }

    pub fn open(&self) -> Result<SC_HANDLE, ServiceError> {
        let handle = unsafe {
            advapi32::OpenServiceW(
                self.manager.handle,
                self.name.encode_utf16_null().as_ptr(),
                winsvc::SERVICE_ALL_ACCESS,
            )
        };

        if handle.is_null() {
            return Err(WindowsService::service_error())
        }

        Ok(handle)
    }

    fn service_error() -> ServiceError {
        match Error::last_os_error().raw_os_error() {
            Some(1056) => return ServiceError::ServiceAlreadyRunning,
            Some(1058) => return ServiceError::ServiceDisabled,
            Some(1060) => return ServiceError::ServiceDoesNotExist,
            Some(1061) => return ServiceError::ServiceCannotAcceptCtrl,
            Some(1062) => return ServiceError::ServiceNotActive,
            Some(1072) => return ServiceError::DeletePending,
            Some(1073) => return ServiceError::ServiceAlreadyExists,
            Some(5) => return ServiceError::AccessViolation,
            Some(6) => return ServiceError::InvalidHandle,
            Some(code) => return ServiceError::UnknownError(code),
            _ => panic!("Can't retrieve OS Error, panicking!")
        }
    }

    pub fn exists(&self) -> bool {
        match self.open() {
            Err(ServiceError::AccessViolation) => {
                println!("INFO: Access violation while opening service.");
                return false
            },
            Err(_) => false,
            Ok(_) => true
        }
    }

    pub fn delete(&self) -> Result<SC_HANDLE, ServiceError> {
        let handle = self.open().expect("Can't open service");
        let success = unsafe { advapi32::DeleteService(handle) == 0 };

        if success {
            return Err(WindowsService::service_error())
        }

        Ok(handle)

    }

    pub fn create(&self) -> Result<SC_HANDLE, ServiceError> {
        let handle = unsafe {
            advapi32::CreateServiceW(
                self.manager.handle,                    // handle
                self.name.encode_utf16_null().as_ptr(),              // service name
                self.name.encode_utf16_null().as_ptr(),              // display name
                winsvc::SERVICE_ALL_ACCESS, // desired access
                SERVICE_KERNEL_DRIVER,      // service type
                SERVICE_DEMAND_START,       // start type
                SERVICE_ERROR_NORMAL,       // error control
                self.path.encode_utf16_null().as_ptr(),              // binary path
                null_mut(),                 // load order
                null_mut(),                 // tag id
                null_mut(),                 // dependencies
                null_mut(),                 // start name
                null_mut(),                 // password
            )
        };

        if handle.is_null() {
            return Err(WindowsService::service_error())
        }

        Ok(handle)
    }

    fn close_service_handle(handle: SC_HANDLE) {
        unsafe {
            advapi32::CloseServiceHandle(handle);
        }
    }
}
