extern crate advapi32;
extern crate winapi;

// external namespaces

use std::ptr::null_mut;
use std::io::Error;
use std::mem;

use winapi::winsvc;
use winapi::winsvc::{SC_HANDLE};

// custom namespaces

#[macro_use]
mod macros;

mod consts;
mod structs;
mod traits;


use traits::EncodeUtf16;
use structs::SERVICE_STATUS_PROCESS;
use consts::{SERVICE_KERNEL_DRIVER, SERVICE_DEMAND_START, SERVICE_ERROR_NORMAL};




#[derive(Debug)]
pub enum ServiceError {
    ServiceAlreadyExists,
    GenericError,
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

    pub fn install(&mut self) {
        if let Err(err) = self.create_service() {
            match err {
                ServiceError::ServiceAlreadyExists => {
                    println!("Failed to install {:?}: Service already exists.", self.name);
                }
                _ => println!("Failed to install {:?}: unknown error {:?}", self.name, err),
            }
        }
    }

    pub fn start(&self) {
        // let handle = self.open().expect("Can't open service");
        unimplemented!();
    }


    fn query_service_status(
        service: SC_HANDLE
    ) -> Result<SERVICE_STATUS_PROCESS, String> {

        let mut process: SERVICE_STATUS_PROCESS = unsafe { mem::zeroed() };
        let mut size: u32 = mem::size_of::<SERVICE_STATUS_PROCESS>() as u32;

        let result = unsafe {
            advapi32::QueryServiceStatusEx(
                service,
                winsvc::SC_STATUS_PROCESS_INFO,
                mem::transmute::<&mut SERVICE_STATUS_PROCESS, *mut u8>(&mut process),
                size,
                &mut size,
            )
        };

        match result == 0 {
            true => return Ok(process),
            false => return Err(Error::last_os_error().to_string())
        }
    }

    pub fn query(&self, service: SC_HANDLE) {

        // let info = WindowsService::query_service_status( service ).expect("Can't query service");
        unimplemented!();
    }

    pub fn open(&self) -> Result<SC_HANDLE, String> {
        let handle = unsafe {
            advapi32::OpenServiceW(
                self.manager.handle,
                self.name.encode_utf16_null().as_ptr(),
                winsvc::SERVICE_ALL_ACCESS,
            )
        };

        if handle.is_null() {
            return Err(Error::last_os_error().to_string());
        }

        Ok(handle)
    }

    pub fn create_service(&self) -> Result<SC_HANDLE, ServiceError> {
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
            match Error::last_os_error().raw_os_error() {
                Some(1073) => return Err(ServiceError::ServiceAlreadyExists),
                _ => return Err(ServiceError::GenericError),
            }
        }

        Ok(handle)
    }

    fn close_service_handle(handle: SC_HANDLE) {
        unsafe {
            advapi32::CloseServiceHandle(handle);
        }
    }
}
