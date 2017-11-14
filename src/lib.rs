extern crate advapi32;
extern crate winapi;

use std::ptr::null_mut;
use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use winapi::winsvc;
use winapi::winsvc::{SC_HANDLE, SC_STATUS_TYPE};
use winapi::minwindef::DWORD;

pub const SERVICE_KERNEL_DRIVER: DWORD = 0x00000001;
pub const SERVICE_DEMAND_START: DWORD = 0x00000003;
pub const SERVICE_ERROR_NORMAL: DWORD = 0x00000001;

#[derive(Debug)]
pub enum ServiceError {
    ServiceAlreadyExists,
    GenericError,
}

fn wide_null_string(data: &str) -> Vec<u16> {
    OsStr::new(data).encode_wide().chain(once(0)).collect()
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
        let handle = self.open_service().expect("Can't open service");
    }


    fn query_service_status(
        service: SC_HANDLE,
        status_type: SC_STATUS_TYPE,
        buffer: &mut [u8],
    ) -> bool {
        let mut size: u32 = buffer.len() as u32;

        let result = unsafe {
            advapi32::QueryServiceStatusEx(
                service,
                status_type,
                buffer.as_mut_ptr(),
                size,
                &mut size,
            )
        };

        result > 0
    }

    pub fn query_service(&self, service: SC_HANDLE) {

        // TODO: construct a structure that matches with SC_STATUS_PROCESS_INFO
        let mut vec = Vec::with_capacity(1000 as usize);

        if WindowsService::query_service_status(
            service,
            winsvc::SC_STATUS_PROCESS_INFO,
            vec.as_mut_slice(),
        )
        {
            WindowsService::close_service_handle(service);
        }
    }

    fn open_service(&self) -> Result<SC_HANDLE, String> {
        let handle = unsafe {
            advapi32::OpenServiceW(
                self.manager.handle,
                wide_null_string(&self.name).as_ptr(),
                winsvc::SERVICE_ALL_ACCESS,
            )
        };

        if handle.is_null() {
            return Err(Error::last_os_error().to_string());
        }

        Ok(handle)
    }

    pub fn create_service(&self) -> Result<SC_HANDLE, ServiceError> {
        let name: Vec<u16> = wide_null_string(&self.name);
        let binary_path: Vec<u16> = wide_null_string(&self.path);

        let handle = unsafe {
            advapi32::CreateServiceW(
                self.manager.handle,                    // handle
                name.as_ptr(),              // service name
                name.as_ptr(),              // display name
                winsvc::SERVICE_ALL_ACCESS, // desired access
                SERVICE_KERNEL_DRIVER,      // service type
                SERVICE_DEMAND_START,       // start type
                SERVICE_ERROR_NORMAL,       // error control
                binary_path.as_ptr(),       // binary path
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
