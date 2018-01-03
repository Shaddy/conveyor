use std::fmt;
use std::sync::Arc;

use super::memory;
use super::iochannel::{Device};
use super::symbols;
use super::symbols::parser::Error;

fn get_offset(target: &str) -> u16 {
    match symbols::parser::find_offset("ntoskrnl.pdb", &target) {
        Err(Error::IoError(_)) => {
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
    pointer: u64,
    list: LinkedList
}

impl Process {
    pub fn new(device: Arc<Device>, pointer: u64) -> Process {
        let offset = get_offset("_EPROCESS.ActiveProcessLinks");

        Process {
            device: device.clone(),
            pointer: pointer,
            list: LinkedList::new(device.clone(), pointer, offset)
        }
    }

    #[allow(dead_code)]
    pub fn backward(&self) -> Process {
        let next = self.list.backward();

        Process {
            device: self.device.clone(),
            pointer: next.ptr(),
            list: next
        }
    }

    pub fn forward(&self) -> Process {
        let next = self.list.forward();

        Process {
            device: self.device.clone(),
            pointer: next.ptr(),
            list: next
        }
    }

    pub fn name(&self) -> String {
        let offset = get_offset("_EPROCESS.ImageFileName");
        let name = memory::read_virtual_memory(&self.device, self.pointer + (offset as u64), 15);
        String::from_utf8(name).expect("can't build process name")
                        .split(|c| c as u8 == 0x00).nth(0).unwrap().to_string()
    }
}

impl Iterator for Process {
    type Item = Process;

    fn next(&mut self) -> Option<Process> {
        let process = self.forward();
        self.pointer = process.pointer;
        self.list = process.list.clone();

        Some(process)
    }
}

impl fmt::Display for Process {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Process(name: {:?}, list: {})", self.name(), self.list)
    }
}
