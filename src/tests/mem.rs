// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;

use std::{fmt};
use super::common;
use super::sentry::{io, memory};
use super::iochannel::{Device};
use super::sentry::memguard::Filter;
use super::sentry::memory::{Map, MapMode};
use super::failure::Error;


use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memory")
                .subcommand(SubCommand::with_name("read"))
                .subcommand(SubCommand::with_name("fuzz-kernel-map-1"))
                .subcommand(SubCommand::with_name("virtual"))
                .subcommand(SubCommand::with_name("write"))
                .subcommand(SubCommand::with_name("kernel-map"))
                .subcommand(SubCommand::with_name("map"))
}

pub fn tests(matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("fuzz-kernel-map-1",      Some(matches))  => test_fuzz_memory(matches, &tx),
        ("virtual",                Some(matches))  => test_virtual_memory(matches, &tx),
        ("write",                  Some(matches))  => test_memory_write(matches, &tx),
        ("map",                    Some(matches))  => test_memory_map(matches, &tx),
        ("kernel-map",             Some(matches))  => test_kernel_map(matches, &tx),
        _                                => Ok(println!("{}", matches.usage()))
    }
}

#[allow(unused_variables)]
fn test_fuzz_memory(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    let _filters: Vec<Filter> = (0..1000).map(|_| Filter::new(&device)).collect();

    Ok(())
}

fn test_kernel_map(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
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

    // debug!(logger, "TestStruct: allocated {} bytes at:
    //                 kernel: 0x{:016x}
    //                 user:   0x{:016x}", map.size(), map.kernel_ptr(), map.as_ptr() as u64);
        ShellMessage::send(&tx, format!("TestStruct: allocated {} bytes at:
                    kernel: 0x{:016x}
                    user:   0x{:016x}", map.size(), map.kernel_ptr(), map.as_ptr() as u64),
                    MessageType::close,0);

    unsafe {
        let test = &mut *map.as_mut_ptr();
        test.first  = 0x1122_3344;
        test.second = 0x5566_7788;
    }

    // debug!(logger, "reading kernel pointer 0x{:016x}", map.kernel_ptr());
    ShellMessage::send(&tx, format!("reading kernel pointer 0x{:016x}", map.kernel_ptr()), MessageType::spinner,1);

    let v = memory::read_virtual_memory(&device, map.kernel_ptr(), map.size())
                            .expect("error reading memory");


    let u: &TestStruct = unsafe{ &*map.as_ptr() };
    let k: &TestStruct = unsafe { &*(v.as_ptr() as *const TestStruct) };

    // debug!(logger, "from-user: {}", u);
    ShellMessage::send(&tx, format!("from-user: {}", u), MessageType::close,2);
    // debug!(logger, "from-kernel: {}", k);
    ShellMessage::send(&tx, format!("from-kernel: {}", k), MessageType::close,3);
    Ok(())
}

fn test_virtual_memory(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    // debug!(logger, "opened sentry: {:?}", device);
        ShellMessage::send(&tx, format!("opened sentry: {:?}", device), MessageType::spinner,0);

    let v = common::dummy_vector(0x200);

    let size = v.len();

    // debug!(logger, "write-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());
        ShellMessage::send(&tx, format!("write-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len()), MessageType::spinner,0);

    let addr = memory::alloc_virtual_memory(&device, size).unwrap();

    // debug!(logger, "alloc_virtual_memory: 0x{:016x}", addr);
        ShellMessage::send(&tx, format!("alloc_virtual_memory: 0x{:016x}", addr), MessageType::spinner,0);

    let written = memory::write_virtual_memory(&device, addr, v).unwrap();

    // debug!(logger, "write_virtual_memory: {} bytes written", written);
        ShellMessage::send(&tx, format!("write_virtual_memory: {} bytes written", written), MessageType::spinner,0);

    let v = memory::read_virtual_memory(&device, addr, size).unwrap();

    // debug!(logger, "reading 0x{:08x} bytes from 0x{:016x}", addr, size);
        ShellMessage::send(&tx, format!("reading 0x{:08x} bytes from 0x{:016x}", addr, size), MessageType::spinner,0);

    // debug!(logger, "read-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());
        ShellMessage::send(&tx, format!("read-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len()), MessageType::spinner,0);

    let output = common::dump_vector(&v);

    // debug!(logger, "{}", output);
        ShellMessage::send(&tx, format!("{}", output), MessageType::spinner,0);

    // debug!(logger, "free_virtual_memory: 0x{:016x}", addr);
        ShellMessage::send(&tx, format!("free_virtual_memory: 0x{:016x}", addr), MessageType::close,0);
    memory::free_virtual_memory(&device, addr).unwrap();
    Ok(())
}

fn test_memory_write(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    let addr = memory::alloc_virtual_memory(&device, 0x200).unwrap();
    // debug!(logger, "reading virtual memory");
        ShellMessage::send(&tx, format!("reading virtual memory"), MessageType::spinner,0);
    let v = memory::read_virtual_memory(&device, addr, 0x200).unwrap();

    // debug!(logger, "writting virtual memory");
        ShellMessage::send(&tx, format!("writting virtual memory"), MessageType::close,0);
    memory::write_virtual_memory(&device, addr, v).unwrap();
    memory::free_virtual_memory(&device, addr).unwrap();
    Ok(())
}


fn test_memory_map(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    let addr = memory::alloc_virtual_memory(&device, 0x200).unwrap();
    let map = Map::new(&device, addr, 0x200, Some(MapMode::UserMode));

    // debug!(logger, "map: {:?}", map);
        ShellMessage::send(&tx, format!("map: {:?}", map), MessageType::close,0);
    memory::free_virtual_memory(&device, addr).unwrap();

    Ok(())
}
