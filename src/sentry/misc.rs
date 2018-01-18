use std::fmt;
use std::sync::Arc;

use super::ffi::traits::EncodeUtf16;
use super::ffi::{NtQuerySystemInformation, SystemInformationClass};

use super::winapi::um::{libloaderapi, processthreadsapi};
use super::{io, memory, symbols, misc};

use std::{ptr, slice};

use super::error::MiscError;
use super::failure::Error;

use std::io::Error as BaseError;

use super::winapi::shared::minwindef::{ LPVOID, 
                                        HMODULE };
use super::iochannel::{Device, IoCtl};
use super::io::{IOCTL_SENTRY_TYPE};

use super::symbols::parser::Error as PdbError;

use super::structs::{RTL_PROCESS_MODULE_INFORMATION, SE_GET_EXPORT_ADDRESS, RawStruct};

pub fn get_offset(target: &str) -> Result<u16, Error> {
    match symbols::parser::find_offset("ntoskrnl.pdb", target) {
        Err(PdbError::IoError(_)) => {
            symbols::downloader::PdbDownloader::new("c:\\windows\\system32\\ntoskrnl.exe".to_string()).download()?;

            Ok(symbols::parser::find_offset("ntoskrnl.pdb", target)?)
        },
        Err(err) => {
            panic!("error parsing PDB {}", err);
        }
        Ok(offset) => Ok(offset)
    }
}


#[derive(Clone)]
pub struct LinkedList {
    device: Arc<Device>,
    offset: u16,
    pointer: u64,
}

impl LinkedList {
    pub fn new(device: Arc<Device>, pointer: u64, offset: u16) -> LinkedList {
        LinkedList {
            device: device,
            offset: offset,
            pointer: pointer + u64::from(offset)
        }
    }

    pub fn ptr(&self) -> u64 {
        self.pointer - u64::from(self.offset)
    }

    #[allow(dead_code)]
    pub fn backward(&self) -> LinkedList {
        let blink = memory::read_u64(&self.device, self.pointer + 8).unwrap();

        LinkedList {
            device: Arc::clone(&self.device),
            offset: self.offset,
            pointer: blink
        }
    }

    pub fn forward(&self) -> LinkedList {
        let flink = memory::read_u64(&self.device, self.pointer).unwrap();

        LinkedList {
            device: Arc::clone(&self.device),
            offset: self.offset,
            pointer: flink
        }
    }
}

impl Iterator for LinkedList {
    type Item = LinkedList;

    fn next(&mut self) -> Option<LinkedList> {
        let next = self.forward();
        self.offset = next.offset;
        self.pointer = next.pointer;
        Some(next)
    }
}

impl fmt::Display for LinkedList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinkedList(flink: 0x{:016x}, blink: 0x{:016x})", self.pointer, self.pointer + 8)
    }
}
impl PartialEq for LinkedList {
    fn eq(&self, other: &LinkedList) -> bool {
        self.pointer == other.pointer
    }
}

#[derive(Clone)]
pub struct Process {
    device: Arc<Device>,
    object: u64,
    list: LinkedList
}

impl Process {
    pub fn current() -> Process {
        let pid = unsafe { processthreadsapi::GetCurrentProcessId() };
        misc::WalkProcess::iter().find(|p| p.id() == u64::from(pid))
                                    .expect("can't find own EPROCESS")
    }
    pub fn system() -> Result<Process, Error> {
        let device = Device::new(io::SE_NT_DEVICE_NAME)?;
        let system_pointer = system_process_pointer(&device)?;
        let addr = memory::read_u64(&device, system_pointer)?;
        Ok(Process::new(Arc::new(device), addr)?)
    }

    pub fn new(device: Arc<Device>, object: u64) -> Result<Process, Error> {
        let target = "_EPROCESS.ActiveProcessLinks";
        let offset = get_offset(target)?;

        Ok(Process {
            device: Arc::clone(&device),
            object: object,
            list: LinkedList::new(Arc::clone(&device), object, offset)
        })
    }

    #[allow(dead_code)]
    pub fn backward(&self) -> Process {
        let next = self.list.backward();

        Process {
            device: Arc::clone(&self.device),
            object: next.ptr(),
            list: next
        }
    }

    pub fn forward(&self) -> Process {
        let next = self.list.forward();

        Process {
            device: Arc::clone(&self.device),
            object: next.ptr(),
            list: next
        }
    }

    pub fn object(&self) -> u64 {
        self.object
    }

    pub fn token(&self) -> u64 {
        let target = "_EPROCESS.Token";
        let offset = get_offset(target).expect(target);
        memory::read_u64(&self.device, self.object + u64::from(offset)).unwrap()
    }

    pub fn id(&self) -> u64 {
        let target = "_EPROCESS.UniqueProcessId";
        let offset = get_offset(target).expect(target);
        memory::read_u64(&self.device, self.object + u64::from(offset)).unwrap()
    }

    pub fn name(&self) -> String {
        let target = "_EPROCESS.ImageFileName";
        let offset = get_offset(target).expect(target);
        let name = memory::read_virtual_memory(&self.device, self.object + u64::from(offset), 15).unwrap();
        String::from_utf8(name).expect("can't build process name")
                        .split(|c| c as u8 == 0x00).nth(0).unwrap().to_string()
    }
}

impl PartialEq for Process {
    fn eq(&self, other: &Process) -> bool {
        self.object == other.object
    }
}

pub struct WalkProcess {
    head: Process,
    curr: Process
}

impl WalkProcess {
    pub fn iter() -> WalkProcess {
        let head = Process::system().expect("can't get system process");
        WalkProcess {
            head: head.clone(),
            curr: head.forward() 
        }
    }
}

impl Iterator for WalkProcess {
    type Item = Process;

    fn next(&mut self) -> Option<Process> {
        let process = self.curr.clone();
        let next = self.curr.forward();

        self.curr = next.clone();

        if next == self.head { return None };
        Some(process)
    }
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Process(name: {:?}, list: {})", self.name(), self.list)
    }
}

pub struct Driver {
    pub name: String,
    pub base: u64,
    pub size: usize
}

impl Driver {
    pub fn size(&self) -> usize {
        self.size
    }

    pub fn base(&self) -> u64 {
        self.base
    }
}

impl fmt::Display for Driver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Driver {{ name: {:?}, base: 0x{:016x} }}", self.name, self.base)
    }
}

pub struct Drivers {
    drivers: Vec<RTL_PROCESS_MODULE_INFORMATION>,
    curr: usize,
    limit: usize
}

impl Drivers {
    pub fn contains(name: &str) -> Option<Driver> {
        Drivers::iter().find(|driver| driver.name.contains(name))
    }

    pub fn iter() -> Drivers {

        // get total size of allocation
        let size = query_system_information_size(SystemInformationClass::SystemModuleInformationEx);

        let mut buffer: Vec<u8> = vec![0; size];

        // fill module information
        let _ = query_system_information(SystemInformationClass::SystemModuleInformationEx, 
                                buffer.as_mut_ptr(),
                                buffer.len());

        let (count, modules) = unsafe { 
            
            let count = *{ buffer.as_ptr() as *const u32 } as usize;
            let modules = slice::from_raw_parts(buffer.as_ptr().offset(8) as *const RTL_PROCESS_MODULE_INFORMATION, count);

            (count, modules)
        };

        Drivers {
            drivers: modules.to_vec(),
            curr: 0,
            limit: count
        }
    }
}

impl Iterator for Drivers {
    type Item = Driver;
    
    fn next(&mut self) -> Option<Driver> {

        if self.curr >= self.limit {
            return None
        }

        let module = self.drivers[self.curr];

        self.curr += 1;

        Some(Driver {
                name: module.FullPathName.iter()
                                         .map(|&c| char::from(c))
                                         .take_while(|&c| c != char::from(0))
                                         .collect::<String>(),
                base: module.ImageBase as u64,
                size: module.ImageSize as usize
        })
    }
}

pub fn query_system_information_size(class: SystemInformationClass) -> usize {
    query_system_information::<u8>(class, ptr::null_mut(), 0)
}

pub fn query_system_information<T>(class: SystemInformationClass, buffer: *mut T, size: usize) -> usize {
    let mut bytes: u32 = 0;

    let _ = unsafe {
            NtQuerySystemInformation(
                class,
                buffer as LPVOID,
                size as u32,
                &mut bytes
            ) == 0
    };

    bytes as usize
}

pub fn list_kernel_drivers() {
    Drivers::iter().for_each(|driver|
        println!("{}", driver)
    );
}

pub fn get_kernel_base() -> u64 {
    Drivers::iter().take(1).nth(0).unwrap().base
}

pub fn load_library(name: &str) -> Result<u64, MiscError> {
    unsafe {
        let value = libloaderapi::LoadLibraryW(name.encode_utf16_null().as_ptr()) as u64;
        if value != 0 {
            Ok(value)
        } else {
            Err(MiscError::LoadLibrary(BaseError::last_os_error().to_string()))
        }
    } 
}

pub fn kernel_export_address(device: &Device, base: u64, name: &str) -> Result<u64, Error> {
    let control = IoCtl::new(Some("SE_GET_EXPORT_ADDRESS"), IOCTL_SENTRY_TYPE, 0x0A62, None, None);

    let mut info = SE_GET_EXPORT_ADDRESS::init();

    info.ModuleBase = base;

    name.chars().enumerate().for_each(|(index, value)| info.Name[index] = value as u8);

    let (ptr, len) = (info.as_ptr(), info.size());

    device.raw_call(control, ptr, len)?;

    Ok(info.Address)
}

pub fn user_proc_addr(base: u64, name: &str) -> Result<u64, MiscError> {
    // for some reason its necessary to do this in order to correctly pass the string
    // at some point the reference to native string breaks the result
    let name = String::from(name);

    unsafe {
        let value = libloaderapi::GetProcAddress(base as HMODULE, name.as_ptr() as *const i8) as u64;
        if value != 0 {
            Ok(value)
        } else {
            Err(MiscError::GetProcedure(BaseError::last_os_error().to_string()))
        }
    }
}

pub fn fixed_procedure_address(base: u64, name: &str, procedure: &str) -> u64 {
    let dynamic_base = load_library(name)
                            .expect(name);

    let address = match user_proc_addr(dynamic_base, procedure) {
        Err(err) => { panic!("{}", err.to_string()) },
        Ok(address) => address
    };

    (address - dynamic_base) + base
}

pub fn system_process_pointer(device: &Device) -> Result<u64, Error> {
    kernel_export_address(device, get_kernel_base(), "PsInitialSystemProcess")
}

#[allow(dead_code)]
#[inline]
pub fn set_breakpoint() {
    unsafe { asm!("int3") };
}