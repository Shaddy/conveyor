use std::fmt;
use std::sync::Arc;

use super::ffi::traits::EncodeUtf16;

use super::winapi::um::{psapi, libloaderapi};
use super::{core, memory, symbols, misc};

use std::io::Error;
use std::mem;

use super::winapi::shared::minwindef::{ DWORD, 
                                        LPVOID, 
                                        HMODULE };
use super::iochannel::{Device};
use super::symbols::parser::Error as PdbError;

fn get_offset(target: &str) -> u16 {
    match symbols::parser::find_offset("ntoskrnl.pdb", &target) {
        Err(PdbError::IoError(_)) => {
            symbols::downloader::PdbDownloader::new("c:\\windows\\system32\\ntoskrnl.exe".to_string()).download()
                                            .expect("Error downloading PDB");

            symbols::parser::find_offset("ntoskrnl.pdb", &target).expect("can't retrieve offset")
        },
        Err(err) => {
            panic!("error parsing PDB {}", err);
        }
        Ok(offset) => offset
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
            pointer: pointer + offset as u64
        }
    }

    pub fn ptr(&self) -> u64 {
        self.pointer - self.offset as u64
    }

    #[allow(dead_code)]
    pub fn backward(&self) -> LinkedList {
        let blink = memory::read_u64(&self.device, self.pointer + 8);

        LinkedList {
            device: self.device.clone(),
            offset: self.offset,
            pointer: blink
        }
    }

    pub fn forward(&self) -> LinkedList {
        let flink = memory::read_u64(&self.device, self.pointer);

        LinkedList {
            device: self.device.clone(),
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

pub struct Process {
    device: Arc<Device>,
    object: u64,
    list: LinkedList
}

impl Process {
    pub fn system() -> Process {
        let device = Device::new(core::SE_NT_DEVICE_NAME);
        let addr = memory::read_u64(&device, misc::system_process_pointer());

        Process::new(Arc::new(device), addr)
    }

    pub fn new(device: Arc<Device>, object: u64) -> Process {
        let offset = get_offset("_EPROCESS.ActiveProcessLinks");

        Process {
            device: device.clone(),
            object: object,
            list: LinkedList::new(device.clone(), object, offset)
        }
    }

    #[allow(dead_code)]
    pub fn backward(&self) -> Process {
        let next = self.list.backward();

        Process {
            device: self.device.clone(),
            object: next.ptr(),
            list: next
        }
    }

    pub fn forward(&self) -> Process {
        let next = self.list.forward();

        Process {
            device: self.device.clone(),
            object: next.ptr(),
            list: next
        }
    }

    pub fn object(&self) -> u64 {
        self.object
    }

    pub fn token(&self) -> u64 {
        let offset = get_offset("_EPROCESS.Token");
        memory::read_u64(&self.device, self.object + offset as u64)
    }

    pub fn id(&self) -> u64 {
        let offset = get_offset("_EPROCESS.UniqueProcessId");
        memory::read_u64(&self.device, self.object + offset as u64)
    }

    pub fn name(&self) -> String {
        let offset = get_offset("_EPROCESS.ImageFileName");
        let name = memory::read_virtual_memory(&self.device, self.object + (offset as u64), 15);
        String::from_utf8(name).expect("can't build process name")
                        .split(|c| c as u8 == 0x00).nth(0).unwrap().to_string()
    }
}

impl Iterator for Process {
    type Item = Process;

    fn next(&mut self) -> Option<Process> {
        let process = self.forward();
        self.object = process.object;
        self.list = process.list.clone();

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
    pub base: u64
}

impl Driver {
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
    drivers: Vec<u64>,
    curr: usize,
    limit: usize
}

impl Drivers {
    pub fn contains(name: &str) -> Option<Driver> {
        Drivers::iter().find(|driver| driver.name.contains(name))
    }

    pub fn iter() -> Drivers {
        let mut needed: DWORD = 0;
        let mut drivers: Vec<u64> = vec![0; 1024];

        let result = unsafe {
            psapi::EnumDeviceDrivers(drivers.as_mut_ptr() as *mut LPVOID, 
                    drivers.len() as u32, 
                    &mut needed)

        };

        if result == 0 {
            panic!(Error::last_os_error().to_string())
        }

        if needed > (drivers.len() * mem::size_of::<usize>()) as u32 {
            panic!(format!("buffer is less than {}", needed))
        }

        Drivers {
            drivers: drivers,
            curr: 0,
            limit: (needed / mem::size_of::<usize>() as u32) as usize
        }
    }
}

impl Iterator for Drivers {
    type Item = Driver;
    
    fn next(&mut self) -> Option<Driver> {
        let mut content: Vec<u16> = vec![0; 1024];
        let base = self.drivers[self.curr];

        if base == 0 { return None } else { self.curr += 1 };

        if self.curr > self.limit { return None };

        let length = unsafe { psapi::GetDeviceDriverBaseNameW(base as LPVOID, content.as_mut_ptr(), 1024 / 2) };
        if length <= 0 { return None } 

        Some(Driver {
                name: String::from_utf16(&content[..length as usize])
                            .expect("failed to parse driver name"),
                base: base
        })
    }
}

pub fn list_kernel_drivers() {
    Drivers::iter().for_each(|driver|
        println!("{}", driver)
    );
}

pub fn get_kernel_base() -> u64 {
    Drivers::iter().take(1).nth(0).unwrap().base
}

pub fn load_library(name: &str) -> Result<u64, String> {
    unsafe {
        let value = libloaderapi::LoadLibraryW(name.encode_utf16_null().as_ptr()) as u64;
        if value != 0 {
            Ok(value)
        } else {
            Err(Error::last_os_error().to_string())
        }
    } 
}

pub fn get_proc_addr(base: u64, name: &str) -> Result<u64, String> {
    // for some reason its necessary to do this in order to correctly pass the string
    // at some point the reference to native string breaks the result
    let name = String::from(name);
    unsafe {
        let value = libloaderapi::GetProcAddress(base as HMODULE, name.as_ptr() as *const i8) as u64;
        if value != 0 {
            Ok(value)
        } else {
            Err(Error::last_os_error().to_string())
        }
    }
}

pub fn fixed_procedure_address(base: u64, name: &str, procedure: &str) -> u64 {
    let dynamic_base = load_library(name)
                            .expect(name);

    let address = get_proc_addr(dynamic_base, procedure)
                            .expect(procedure);

    (address - dynamic_base) + base
}

pub fn system_process_pointer() -> u64 {
    fixed_procedure_address(get_kernel_base(), "ntoskrnl.exe", "PsInitialSystemProcess")
}