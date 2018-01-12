// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;

use std::{fmt};
use super::common;
use super::sentry::{io, memory};
use super::iochannel::{Device};
use super::sentry::memory::{Map, MapMode};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memory")
                .subcommand(SubCommand::with_name("read"))
                .subcommand(SubCommand::with_name("virtual"))
                .subcommand(SubCommand::with_name("write"))
                .subcommand(SubCommand::with_name("kernel-map"))
                .subcommand(SubCommand::with_name("map"))
}

pub fn tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("virtual",      Some(matches))  => test_virtual_memory(matches, logger),
        ("write",        Some(matches))  => test_memory_write(matches, logger),
        ("map",          Some(matches))  => test_memory_map(matches, logger),
        ("kernel-map",   Some(matches))  => test_kernel_map(matches, logger),
        _                                => println!("{}", matches.usage())
    }
}

fn test_kernel_map(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    struct TestStruct {
        first:  u64,
        second: u64, 
    }

    impl fmt::Display for TestStruct {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "TestStruct {{ first: 0x{:016x}, second: 0x{:016x} }}", self.first, self.second)
        }
    }

    let map = memory::KernelAlloc::<TestStruct>::new(&device);

    debug!(logger, "TestStruct: allocated {} bytes at:
                    kernel: 0x{:016x}
                    user:   0x{:016x}", map.size(), map.kernel_ptr(), map.as_ptr() as u64);

    unsafe {
        let test = &mut *map.as_mut_ptr();
        test.first = 0x11223344;
        test.second = 0x55667788;
    }

    debug!(logger, "reading kernel pointer 0x{:016x}", map.kernel_ptr());

    let v = memory::read_virtual_memory(&device, map.kernel_ptr(), map.size())
                            .expect("error reading memory");


    let u: &TestStruct = unsafe{ &*map.as_ptr() };
    let k: &TestStruct = unsafe { &*(v.as_ptr() as *const TestStruct) };

    debug!(logger, "from-user: {}", u);
    debug!(logger, "from-kernel: {}", k);
}

fn test_virtual_memory(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    debug!(logger, "opened sentry: {:?}", device);

    let v = common::dummy_vector(0x200);

    let size = v.len();

    debug!(logger, "write-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());

    let addr = memory::alloc_virtual_memory(&device, size).unwrap();

    debug!(logger, "alloc_virtual_memory: 0x{:016x}", addr);

    let written = memory::write_virtual_memory(&device, addr, v).unwrap();

    debug!(logger, "write_virtual_memory: {} bytes written", written);

    let v = memory::read_virtual_memory(&device, addr, size).unwrap();

    debug!(logger, "reading 0x{:08x} bytes from 0x{:016x}", addr, size);

    debug!(logger, "read-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());

    let output = common::dump_vector(v);
    
    debug!(logger, "{}", output);

    debug!(logger, "free_virtual_memory: 0x{:016x}", addr);
    memory::free_virtual_memory(&device, addr).unwrap();
}

fn test_memory_write(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    let addr = memory::alloc_virtual_memory(&device, 0x200).unwrap();
    debug!(logger, "reading virtual memory");
    let v = memory::read_virtual_memory(&device, addr, 0x200).unwrap();

    debug!(logger, "writting virtual memory");
    memory::write_virtual_memory(&device, addr, v).unwrap();
    memory::free_virtual_memory(&device, addr).unwrap();
}


fn test_memory_map(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    let addr = memory::alloc_virtual_memory(&device, 0x200).unwrap();
    let map = Map::new(&device, addr, 0x200, Some(MapMode::UserMode));

    debug!(logger, "map: {:?}", map);
    memory::free_virtual_memory(&device, addr).unwrap();

}
