extern crate advapi32;
extern crate winapi;

// external namespaces

use std::ptr::null_mut;
use std::io::Error;
use std::mem::{transmute, zeroed, size_of_val};

use winapi::winsvc;
use winapi::winsvc::{SC_HANDLE};

// custom namespaces

#[macro_use]
mod macros;

#[macro_use]
extern crate bitflags;


mod consts;
pub mod structs;
mod traits;


use traits::EncodeUtf16;
use structs::{SERVICE_STATUS_PROCESS, ServiceInfo};
use consts::{SERVICE_KERNEL_DRIVER, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL};


#[derive(Debug, PartialEq)]
pub enum ServiceError {
    DeletePending,
    ServiceAlreadyExists,
    ServiceDoesNotExist,
    AccessViolation,
    InvalidHandle,
    UnknownError(i32),
}

#[derive(Debug)]
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

#[derive(Debug)]
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

    pub fn remove(&mut self) {
        self.delete().expect("Can't remove service");
        println!("Service {:?} has been successfully removed", self.name);
    }

    pub fn install(&mut self) {
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
    }

    pub fn stop(&self) {
        let _service = self.open().expect("Unable to open service");

        let result = false;

        // In stop case we should call to ControlService functions for each service dependencie.

        if !result {
            println!("Can't stop service ({}): {:?}", self.name, WindowsService::service_error())
        }
    }

    pub fn start(&self) {
        let _service = self.open().expect("Unable to open service");

        // TODO: Find or implement StartServiceW
        let result = false;
        // let result = unsafe {
        //     advapi32::StartServiceW(
        //         service,
        //         0,
        //         null_mut(),
        //     )
        // };

        if !result {
            println!("Can't start service ({}): {:?}", self.name, WindowsService::service_error())
        }
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


    pub fn query(&self, service: SC_HANDLE) -> ServiceInfo {

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
            Some(1072) => return ServiceError::DeletePending,
            Some(1073) => return ServiceError::ServiceAlreadyExists,
            Some(1060) => return ServiceError::ServiceDoesNotExist,
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
